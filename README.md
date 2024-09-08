# Hasky

[![Commitizen friendly](https://img.shields.io/badge/commitizen-friendly-brightgreen.svg)](http://commitizen.github.io/cz-cli/)

Hasky is a Haskell grading module for [Moody](https://github.com/arghyadipchak/moody)

```sh
Usage: hasky [OPTIONS] --input-file <INPUT_FILE> --test-cases <TEST_CASES>

Options:
  -i, --input-file <INPUT_FILE>
          Input file (moody compatible)
  -o, --output-file <OUTPUT_FILE>
          Output file [default: {input-file}_graded.yml]
  -m, --max-grade <MAX_GRADE>
          Maximum grade (equally divided among testcases) [default: assignment max grade]
  -n, --not-graded
          Do not mark as graded
  -f, --test-functions <TEST_FUNCTIONS>
          File containing testcases helper functions
  -c, --test-cases <TEST_CASES>
          File containing testcases
  -t, --timeout <TIMEOUT>
          Timeout for each test case (in secs)
  -w, --workers <WORKERS>
          No of workers to run [default: 1]
  -h, --help
          Print help

```
