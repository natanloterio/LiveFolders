#!/usr/bin/env bash
cd "$(dirname "$0")"

mkdir -p results

echo "# Experiment Results — $(date +%Y-%m-%d)" > results/summary.md
echo "" >> results/summary.md

failed=()

for d in criteria/*/; do
  if [ -f "$d/run.sh" ]; then
    echo "=== $d ===" | tee -a results/summary.md
    if ! bash "$d/run.sh" 2>&1 | tee -a results/summary.md; then
      failed+=("$d")
    fi
    echo "" | tee -a results/summary.md
  fi
done

if [ ${#failed[@]} -gt 0 ]; then
  echo "## Failed scripts" | tee -a results/summary.md
  for f in "${failed[@]}"; do
    echo "  - $f" | tee -a results/summary.md
  done
fi

echo "Results written to results/summary.md"
