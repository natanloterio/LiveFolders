# ModixFS — Histórico de Conversa

**Data:** 2026-05-18

---

## 1. Ideia Inicial: Alternativa ao MCP via Virtual Filesystem

**Conceito:** Criar uma alternativa ao Model Context Protocol (MCP) usando virtualização de filesystem. Em vez do wrapping tax do MCP (JSON-RPC, schemas, protocol overhead), aproveitar que LLMs já sabem operar via linha de comando.

**Proposta central:**
- Diretório `/tools/` virtual com arquivos representando cada tool instalada
- Cada tool é um arquivo virtual com `index.md` contendo instruções de uso
- LLM usa `cat`, `echo`, pipes — interface que já domina nativamente

---

## 2. Brainstorm: Dimensões do Problema

### O Wrapping Tax do MCP
| Dimensão | MCP | ModixFS |
|----------|-----|---------|
| Protocolo | JSON-RPC | File I/O |
| Discovery | Tool list API | `ls` / `cat` |
| Documentação | Schema descritivo | Markdown livre, dinâmico |
| Invocação | Function call | File write |
| Resultado | JSON response | File read |
| Composição | Limitada | Shell pipes naturais |

### Modelos de Interação Explorados

**Read/Write Síncrono:**
```bash
echo '{"query": "auth bug"}' > /tools/github/search
cat /tools/github/search  # retorna resultados
```

**Pipe + Streams:**
```bash
cat /tools/github/search <<'EOF' | /tools/linear/create_issue
{"query": "auth"}
EOF
```

**Diretório de Jobs (async):**
```bash
echo '{"cmd": "deploy"}' > /tools/vercel/jobs/new
cat /tools/vercel/jobs/abc123/status   # "running" | "complete"
```

**Tool-as-Executable:**
```bash
/tools/github/search --query "auth" --repo "owner/repo"
```

### Estrutura do Filesystem Virtual
```
/tools/
├── index.md
├── github/
│   ├── index.md
│   ├── search
│   └── create_issue
├── linear/
├── compose/        ← LLM cria tools novas como shell scripts
└── .context/       ← estado da sessão
```

### Propriedades Emergentes
- **Auto-documentação**: `index.md` gerado dinamicamente por uso
- **Tool Composition**: LLM cria scripts em `/tools/compose/` combinando tools existentes
- **Permissões como segurança**: `chmod 000 /tools/payments/refund` requer aprovação humana
- **Versionamento**: `/tools/stripe@v2/charge`

---

## 3. Implementação Técnica

### Arquitetura Central

```
LLM Agent
    │ syscalls (read/write/readdir)
ModixFS Layer
    ├── Virtual File Router  (path → tool mapping)
    └── Tool Registry        (read/write/list handlers + session state)
            │
    Tool Implementations
    (GitHub, Linear, Slack...)
```

### Máquina de Estado por Arquivo

```
IDLE (docs) → write(params) → PENDING (exec) → COMPLETE (result)
     ▲                                               │
     └──────────── read() reseta ───────────────────┘
```

### Stack Recomendado
**Rust com `fuser` crate** — performance, segurança, async nativo com `tokio`, sem GIL.

### Interface Core (Rust)
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn documentation(&self) -> String;  // markdown quando LLM faz cat (idle)
    async fn invoke(&self, input: &str, session: &Session) -> ToolResult;
    fn children(&self) -> Vec<String> { vec![] }
}
```

### Async para Tools Longas: Padrão Jobs
```
/tools/vercel/
├── deploy           ← write inicia o job
└── jobs/
    └── <job-id>/
        ├── status   ← "running" | "complete" | "failed"
        ├── logs     ← append-only, streaming
        └── result   ← disponível quando complete
```

### Estrutura do Projeto
```
modixfs/
├── src/
│   ├── fs/         # FUSE impl (Filesystem trait)
│   ├── registry/   # ToolRegistry + Tool trait + session
│   └── tools/      # GitHub, Linear, Slack, shell scripts
└── tools.yaml      # config
```

---

## 4. Windows e Docker

### Windows: Alternativas ao FUSE

| Opção | Descrição | Requer instalação? |
|-------|-----------|-------------------|
| **ProjFS** (Windows Projected FS) | Built-in Windows 10 1809+, sem driver, sem privilégios. Usado pelo VFS for Git. | Não |
| **WinFsp** | Emula API FUSE no Windows, compatível com código existente | Sim (usuário instala) |
| **LD_PRELOAD / DLL Injection** | Intercepta libc sem envolver kernel | Não |

### Docker + SYS_ADMIN: Blocker Real

`SYS_ADMIN` é bloqueado por default em Kubernetes, Cloud Run, Fargate, Azure Container Apps e ambientes enterprise.

**Saída no Linux 5.x+:** FUSE dentro de user namespaces sem `SYS_ADMIN`:
```bash
docker run --device /dev/fuse modixfs  # sem SYS_ADMIN
```
Ainda pode ser bloqueado em ambientes gerenciados. **Para escala real, FUSE não é a resposta em produção.**

### Dois Modos de Deploy

```
Dev / Local                    Prod / Container
────────────────               ──────────────────────
FUSE mount real                In-Process VFS
(naturalidade total)           (zero capabilities)

cat /tools/gh/search           agent.read("/tools/gh/search")
echo '...' > /tools/..         agent.write("/tools/...", data)
```

### Docker como Sandbox de Segurança para LLM

Modelo de segurança por capabilities:
- LLM confinado no container com acesso apenas a `/tools/`
- Sem acesso a filesystem do host, `/etc`, binários arbitrários
- Credenciais apenas no sidecar — LLM nunca as vê
- **Mais seguro que MCP atual**

```yaml
services:
  agent:
    network_mode: none          # sem acesso direto à rede
    environment:
      - MODIXFS_MODE=inprocess
      - MODIXFS_SOCKET=/run/modixfs.sock

  modixfs-sidecar:              # só ele tem acesso à rede externa
    environment:
      - GITHUB_TOKEN=${GITHUB_TOKEN}
      - LINEAR_API_KEY=${LINEAR_API_KEY}
```

### Tabela de Escolhas por Ambiente

| Cenário | Backend | Capabilities |
|---------|---------|-------------|
| Dev local Linux/macOS | FUSE | Nenhuma |
| Dev local Windows | ProjFS | Nenhuma |
| Container Docker prod | In-Process VFS | Nenhuma |
| Kubernetes | In-Process VFS + sidecar | Nenhuma |
| VM (Kata, Firecracker) | virtio-fs | Nenhuma |

---

## 5. O que é FUSE

**FUSE = Filesystem in Userspace**

Mecanismo do Linux/macOS que permite criar filesystems customizados em espaço de usuário, sem privilégios de kernel. O kernel redireciona syscalls (`read`, `write`, `readdir`) do processo consumidor para o programa FUSE.

```
cat /tools/search  →  kernel FUSE module  →  ModixFS process  →  retorna bytes
```

Projetos conhecidos: `sshfs`, `s3fs`, `gocryptfs`.

---

## 6. Por que Rust é Recomendado

### Razão 1: FUSE exige controle de memória preciso

O FUSE kernel module chama seu código diretamente para responder syscalls. Cada `read`, `write`, `readdir` precisa retornar buffers corretos, sem leaks, sem uso-após-free. Linguagens com GC (Go, Java, Python) podem pausar no momento errado — dentro de uma syscall. Rust garante correção de memória **em tempo de compilação**, sem GC e sem runtime.

### Razão 2: Async nativo sem overhead

Tools de LLM são I/O-bound: chamadas HTTP para GitHub, Linear, Slack. O `tokio` do Rust é o runtime async mais performático disponível. Zero-cost abstractions — async não adiciona overhead em runtime.

### Razão 3: Binário único, sem dependências

```bash
# Rust: download e funciona
curl -L https://github.com/org/modixfs/releases/latest/modixfs -o /usr/local/bin/modixfs

# Python: requer Python 3.11+, venv, pip...
# Go: binário estático bom, mas CGO com FUSE complica cross-compilation
```

### Comparativo de Alternativas

| Linguagem | Problema para este projeto |
|-----------|---------------------------|
| **C** | Sem abstrações seguras; memory bugs são certos |
| **Go** | GC pausas em syscall handlers; CGO necessário para FUSE |
| **Python** | GIL impede paralelismo real; deployment complexo |
| **Node.js** | Não é a linguagem certa para código de sistema |
| **Zig** | Ecossistema ainda não maduro para FUSE + async HTTP |

### Ressalva Honesta

Se a equipe não tem experiência com Rust, a curva de aprendizado é real (especialmente o borrow checker). Nesse caso:

- **Go** é a segunda melhor opção: binário estático, concorrência boa, [`bazil.org/fuse`](https://bazil.org/fuse) funcional
- **Python + `fusepy`** para MVP de validação de conceito — roda em dias, migra para Rust depois

---

## Próximos Passos Identificados

- [ ] Escolher tier de implementação do MVP: FUSE real vs In-Process
- [ ] Definir formato de invocação: JSON, YAML, natural language, CLI args
- [ ] Implementar Tool Registry básico em Rust
- [ ] Implementar 2 tools iniciais: filesystem passthrough + GitHub search
- [ ] Validar modelo de interação com Claude Code como LLM de teste
- [ ] Definir estratégia de config (`tools.yaml`) e CLI (`modixfs mount`)
