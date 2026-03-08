# CLOV Token Savings Audit Guide

Complete guide to analyzing your clov token savings with temporal breakdowns and data exports.

## Overview

The `clov pulse` command provides comprehensive analytics for tracking your token savings across time periods.

**Database Location**: `~/.local/share/clov/history.db`
**Retention Policy**: 90 days
**Scope**: Global across all projects, worktrees, and Claude sessions

## Quick Reference

```bash
# Default summary view
clov pulse

# Temporal breakdowns
clov pulse --daily          # All days since tracking started
clov pulse --weekly         # Aggregated by week
clov pulse --monthly        # Aggregated by month
clov pulse --all            # Show all breakdowns at once

# Export formats
clov pulse --all --format json > savings.json
clov pulse --all --format csv > savings.csv

# Combined flags
clov pulse --graph --history --quota    # Classic view with extras
clov pulse --daily --weekly --monthly   # Multiple breakdowns
```

## Command Options

### Temporal Flags

| Flag | Description | Output |
|------|-------------|--------|
| `--daily` | Day-by-day breakdown | All days with full metrics |
| `--weekly` | Week-by-week breakdown | Aggregated by Sunday-Saturday weeks |
| `--monthly` | Month-by-month breakdown | Aggregated by calendar month |
| `--all` | All time breakdowns | Daily + Weekly + Monthly combined |

### Classic Flags (still available)

| Flag | Description |
|------|-------------|
| `--graph` | ASCII graph of last 30 days |
| `--history` | Recent 10 commands |
| `--quota` | Monthly quota analysis (Pro/5x/20x tiers) |
| `--tier <TIER>` | Quota tier: pro, 5x, 20x (default: 20x) |

### Export Formats

| Format | Flag | Use Case |
|--------|------|----------|
| `text` | `--format text` (default) | Terminal display |
| `json` | `--format json` | Programmatic analysis, APIs |
| `csv` | `--format csv` | Excel, data analysis, plotting |

## Output Examples

### Daily Breakdown

```
📅 Daily Breakdown (3 days)
════════════════════════════════════════════════════════════════
Date            Cmds      Input     Output      Saved   Save%
────────────────────────────────────────────────────────────────
2026-01-28        89     380.9K      26.7K     355.8K   93.4%
2026-01-29       102     894.5K      32.4K     863.7K   96.6%
2026-01-30         5        749         55        694   92.7%
────────────────────────────────────────────────────────────────
TOTAL            196       1.3M      59.2K       1.2M   95.6%
```

**Metrics explained:**
- **Cmds**: Number of clov commands executed
- **Input**: Estimated tokens from raw command output
- **Output**: Actual tokens after clov filtering
- **Saved**: Input - Output (tokens prevented from reaching LLM)
- **Save%**: Percentage reduction (Saved / Input × 100)

### Weekly Breakdown

```
📊 Weekly Breakdown (1 weeks)
════════════════════════════════════════════════════════════════════════
Week                      Cmds      Input     Output      Saved   Save%
────────────────────────────────────────────────────────────────────────
01-26 → 02-01              196       1.3M      59.2K       1.2M   95.6%
────────────────────────────────────────────────────────────────────────
TOTAL                      196       1.3M      59.2K       1.2M   95.6%
```

**Week definition**: Sunday to Saturday (ISO week starting Sunday at 00:00)

### Monthly Breakdown

```
📆 Monthly Breakdown (1 months)
════════════════════════════════════════════════════════════════
Month         Cmds      Input     Output      Saved   Save%
────────────────────────────────────────────────────────────────
2026-01        196       1.3M      59.2K       1.2M   95.6%
────────────────────────────────────────────────────────────────
TOTAL          196       1.3M      59.2K       1.2M   95.6%
```

**Month format**: YYYY-MM (calendar month)

### JSON Export

```json
{
  "summary": {
    "total_commands": 196,
    "total_input": 1276098,
    "total_output": 59244,
    "total_saved": 1220217,
    "avg_savings_pct": 95.62
  },
  "daily": [
    {
      "date": "2026-01-28",
      "commands": 89,
      "input_tokens": 380894,
      "output_tokens": 26744,
      "saved_tokens": 355779,
      "savings_pct": 93.41
    }
  ],
  "weekly": [...],
  "monthly": [...]
}
```

**Use cases:**
- API integration
- Custom dashboards
- Automated reporting
- Data pipeline ingestion

### CSV Export

```csv
# Daily Data
date,commands,input_tokens,output_tokens,saved_tokens,savings_pct
2026-01-28,89,380894,26744,355779,93.41
2026-01-29,102,894455,32445,863744,96.57

# Weekly Data
week_start,week_end,commands,input_tokens,output_tokens,saved_tokens,savings_pct
2026-01-26,2026-02-01,196,1276098,59244,1220217,95.62

# Monthly Data
month,commands,input_tokens,output_tokens,saved_tokens,savings_pct
2026-01,196,1276098,59244,1220217,95.62
```

**Use cases:**
- Excel analysis
- Python/R data science
- Google Sheets dashboards
- Matplotlib/seaborn plotting

## Analysis Workflows

### Weekly Progress Tracking

```bash
# Generate weekly report every Monday
clov pulse --weekly --format csv > reports/week-$(date +%Y-%W).csv

# Compare this week vs last week
clov pulse --weekly | tail -3
```

### Monthly Cost Analysis

```bash
# Export monthly data for budget review
clov pulse --monthly --format json | jq '.monthly[] |
  {month, saved_tokens, quota_pct: (.saved_tokens / 6000000 * 100)}'
```

### Data Science Analysis

```python
import pandas as pd
import subprocess

# Get CSV data
result = subprocess.run(['clov', 'pulse', '--all', '--format', 'csv'],
                       capture_output=True, text=True)

# Parse daily data
lines = result.stdout.split('\n')
daily_start = lines.index('# Daily Data') + 2
daily_end = lines.index('', daily_start)
daily_df = pd.read_csv(pd.StringIO('\n'.join(lines[daily_start:daily_end])))

# Plot savings trend
daily_df['date'] = pd.to_datetime(daily_df['date'])
daily_df.plot(x='date', y='savings_pct', kind='line')
```

### Excel Analysis

1. Export CSV: `clov pulse --all --format csv > clov-data.csv`
2. Open in Excel
3. Create pivot tables:
   - Daily trends (line chart)
   - Weekly totals (bar chart)
   - Savings % distribution (histogram)

### Dashboard Creation

```bash
# Generate dashboard data daily via cron
0 0 * * * clov pulse --all --format json > /var/www/dashboard/clov-stats.json

# Serve with static site
cat > index.html <<'EOF'
<script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
<canvas id="savings"></canvas>
<script>
fetch('clov-stats.json')
  .then(r => r.json())
  .then(data => {
    new Chart(document.getElementById('savings'), {
      type: 'line',
      data: {
        labels: data.daily.map(d => d.date),
        datasets: [{
          label: 'Daily Savings %',
          data: data.daily.map(d => d.savings_pct)
        }]
      }
    });
  });
</script>
EOF
```

## Understanding Token Savings

### Token Estimation

clov estimates tokens using `text.len() / 4` (4 characters per token average).

**Accuracy**: ±10% compared to actual LLM tokenization (sufficient for trends).

### Savings Calculation

```
Input Tokens    = estimate_tokens(raw_command_output)
Output Tokens   = estimate_tokens(clov_filtered_output)
Saved Tokens    = Input - Output
Savings %       = (Saved / Input) × 100
```

### Typical Savings by Command

| Command | Typical Savings | Mechanism |
|---------|----------------|-----------|
| `clov git status` | 77-93% | Compact stat format |
| `clov eslint` | 84% | Group by rule |
| `clov vitest run` | 94-99% | Show failures only |
| `clov scan` | 75% | Tree format |
| `clov pnpm list` | 70-90% | Compact dependencies |
| `clov search` | 70% | Truncate + group |

## Database Management

### Inspect Raw Data

```bash
# Location
ls -lh ~/.local/share/clov/history.db

# Schema
sqlite3 ~/.local/share/clov/history.db ".schema"

# Recent records
sqlite3 ~/.local/share/clov/history.db \
  "SELECT timestamp, clov_cmd, saved_tokens FROM commands
   ORDER BY timestamp DESC LIMIT 10"

# Total database size
sqlite3 ~/.local/share/clov/history.db \
  "SELECT COUNT(*),
          SUM(saved_tokens) as total_saved,
          MIN(DATE(timestamp)) as first_record,
          MAX(DATE(timestamp)) as last_record
   FROM commands"
```

### Backup & Restore

```bash
# Backup
cp ~/.local/share/clov/history.db ~/backups/clov-history-$(date +%Y%m%d).db

# Restore
cp ~/backups/clov-history-20260128.db ~/.local/share/clov/history.db

# Export for migration
sqlite3 ~/.local/share/clov/history.db .dump > clov-backup.sql
```

### Cleanup

```bash
# Manual cleanup (older than 90 days)
sqlite3 ~/.local/share/clov/history.db \
  "DELETE FROM commands WHERE timestamp < datetime('now', '-90 days')"

# Reset all data
rm ~/.local/share/clov/history.db
# Next clov command will recreate database
```

## Integration Examples

### GitHub Actions CI/CD

```yaml
# .github/workflows/clov-stats.yml
name: CLOV Stats Report
on:
  schedule:
    - cron: '0 0 * * 1'  # Weekly on Monday
jobs:
  stats:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install clov
        run: cargo install --path .
      - name: Generate report
        run: |
          clov pulse --weekly --format json > stats/week-$(date +%Y-%W).json
      - name: Commit stats
        run: |
          git add stats/
          git commit -m "Weekly clov stats"
          git push
```

### Slack Bot

```python
import subprocess
import json
import requests

def send_clov_stats():
    result = subprocess.run(['clov', 'pulse', '--format', 'json'],
                           capture_output=True, text=True)
    data = json.loads(result.stdout)

    message = f"""
    📊 *CLOV Token Savings Report*

    Total Saved: {data['summary']['total_saved']:,} tokens
    Savings Rate: {data['summary']['avg_savings_pct']:.1f}%
    Commands: {data['summary']['total_commands']}
    """

    requests.post(SLACK_WEBHOOK_URL, json={'text': message})
```

## Troubleshooting

### No data showing

```bash
# Check if database exists
ls -lh ~/.local/share/clov/history.db

# Check record count
sqlite3 ~/.local/share/clov/history.db "SELECT COUNT(*) FROM commands"

# Run a tracked command to generate data
clov git status
```

### Export fails

```bash
# Check for pipe errors
clov pulse --format json 2>&1 | tee /tmp/clov-debug.log | jq .

# Use release build to avoid warnings
cargo build --release
./target/release/clov pulse --format json
```

### Incorrect statistics

Token estimation is a heuristic. For precise measurements:

```bash
# Install tiktoken
pip install tiktoken

# Validate estimation
clov git status > output.txt
python -c "
import tiktoken
enc = tiktoken.get_encoding('cl100k_base')
text = open('output.txt').read()
print(f'Actual tokens: {len(enc.encode(text))}')
print(f'clov estimate: {len(text) // 4}')
"
```

## Best Practices

1. **Regular Exports**: `clov pulse --all --format json > monthly-$(date +%Y%m).json`
2. **Trend Analysis**: Compare week-over-week savings to identify optimization opportunities
3. **Command Profiling**: Use `--history` to see which commands save the most
4. **Backup Before Cleanup**: Always backup before manual database operations
5. **CI Integration**: Track savings across team in shared dashboards

## See Also

- [README.md](../README.md) - Full clov documentation
- [CLAUDE.md](../CLAUDE.md) - Claude Code integration guide
- [ARCHITECTURE.md](../ARCHITECTURE.md) - Technical architecture
