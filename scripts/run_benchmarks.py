#!/usr/bin/env python3
"""
Cross-platform benchmark runner for jsonrepair.

Runs Python and Rust benchmarks with configurable profiles and generates
a comparison table.

Usage:
    python scripts/run_benchmarks.py [profile]
    
Profiles:
    quick    - Fast iteration for development (~30s-1min)
    standard - Balanced accuracy and speed (~2-3min) [DEFAULT]
    heavy    - Maximum accuracy for official benchmarks (~5-8min)
    custom   - Use environment variables or command-line args
"""

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, Optional

# ANSI color codes for better output
class Colors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'

    @staticmethod
    def disable():
        """Disable colors on Windows if not supported."""
        Colors.HEADER = ''
        Colors.OKBLUE = ''
        Colors.OKCYAN = ''
        Colors.OKGREEN = ''
        Colors.WARNING = ''
        Colors.FAIL = ''
        Colors.ENDC = ''
        Colors.BOLD = ''
        Colors.UNDERLINE = ''

# Check if we're on Windows and colors are supported
if os.name == 'nt':
    try:
        import colorama
        colorama.init()
    except ImportError:
        Colors.disable()

# Profile configurations
PROFILES = {
    'quick': {
        'py_min_bytes': 524288,      # 512 KiB
        'py_target_sec': 0.5,
        'py_warmup': 3,
        'jr_min_bytes': 524288,
        'jr_meas_sec': 2,
        'jr_warmup_sec': 1,
        'jr_sample_size': 10,        # Criterion requires >= 10
        'est_time': '1-2min',
        'description': 'Fast iteration for development'
    },
    'standard': {
        'py_min_bytes': 1048576,     # 1 MiB
        'py_target_sec': 1.0,
        'py_warmup': 3,
        'jr_min_bytes': 1048576,
        'jr_meas_sec': 3,
        'jr_warmup_sec': 1,
        'jr_sample_size': 10,        # Criterion requires >= 10
        'est_time': '3-5min',
        'description': 'Balanced accuracy and speed'
    },
    'heavy': {
        'py_min_bytes': 2097152,     # 2 MiB
        'py_target_sec': 2.0,
        'py_warmup': 5,
        'jr_min_bytes': 2097152,
        'jr_meas_sec': 5,
        'jr_warmup_sec': 2,
        'jr_sample_size': 20,        # More samples for better accuracy
        'est_time': '8-12min',
        'description': 'Maximum accuracy for official benchmarks'
    }
}

def print_header(text: str):
    """Print a formatted header."""
    print(f"\n{Colors.BOLD}{Colors.OKCYAN}{'='*60}{Colors.ENDC}")
    print(f"{Colors.BOLD}{Colors.OKCYAN}{text}{Colors.ENDC}")
    print(f"{Colors.BOLD}{Colors.OKCYAN}{'='*60}{Colors.ENDC}\n")

def print_success(text: str):
    """Print a success message."""
    print(f"{Colors.OKGREEN}✓ {text}{Colors.ENDC}")

def print_error(text: str):
    """Print an error message."""
    print(f"{Colors.FAIL}✗ ERROR: {text}{Colors.ENDC}", file=sys.stderr)

def print_info(text: str):
    """Print an info message."""
    print(f"{Colors.OKBLUE}[run] {text}{Colors.ENDC}")

def check_python():
    """Check if Python is available."""
    try:
        result = subprocess.run([sys.executable, '--version'], 
                              capture_output=True, text=True, check=True)
        print_info(f"Python: {result.stdout.strip()}")
        return True
    except Exception as e:
        print_error(f"Python check failed: {e}")
        return False

def check_cargo():
    """Check if Cargo is available."""
    try:
        result = subprocess.run(['cargo', '--version'], 
                              capture_output=True, text=True, check=True)
        print_info(f"Cargo: {result.stdout.strip()}")
        return True
    except Exception as e:
        print_error(f"Cargo not found: {e}")
        return False

def run_python_benchmark(config: Dict, output_file: str = 'python_bench.json') -> bool:
    """Run Python benchmark."""
    print_header("Step 1/3: Running Python benchmark")
    
    args = [
        sys.executable,
        'scripts/py_bench.py',
        '--min-bytes', str(config['py_min_bytes']),
        '--target-sec', str(config['py_target_sec']),
        '--warmup', str(config['py_warmup'])
    ]
    
    print_info(f"Command: {' '.join(args)}")
    print()
    
    try:
        with open(output_file, 'w', encoding='utf-8') as f:
            result = subprocess.run(args, stdout=f, stderr=None, check=True)
        print()
        print_success("Python benchmark completed")
        return True
    except subprocess.CalledProcessError as e:
        print_error(f"Python benchmark failed with exit code {e.returncode}")
        return False
    except Exception as e:
        print_error(f"Python benchmark failed: {e}")
        return False

def run_rust_benchmark(config: Dict) -> bool:
    """Run Rust benchmarks (container, stream, writer)."""
    print_header("Step 2/3: Running Rust benchmarks (container/stream/writer)")
    print_info("This may take a while, please be patient...")
    print()
    
    # Set environment variables
    env = os.environ.copy()
    env['JR_MIN_BYTES'] = str(config['jr_min_bytes'])
    env['JR_MEAS_SEC'] = str(config['jr_meas_sec'])
    env['JR_WARMUP_SEC'] = str(config['jr_warmup_sec'])
    env['JR_SAMPLE_SIZE'] = str(config['jr_sample_size'])
    # Use a separate target dir to avoid Windows linker file locks on existing bench executables
    env.setdefault('CARGO_TARGET_DIR', 'target_bench')
    
    print_info(f"Environment: JR_MIN_BYTES={env['JR_MIN_BYTES']}, "
              f"JR_MEAS_SEC={env['JR_MEAS_SEC']}, "
              f"JR_WARMUP_SEC={env['JR_WARMUP_SEC']}, "
              f"JR_SAMPLE_SIZE={env['JR_SAMPLE_SIZE']}")
    print()
    
    try:
        # container benches (including valid_json)
        bench_args = ['--no-default-features', '--features', 'serde']
        subprocess.run(['cargo', 'bench', '--bench', 'container_bench', *bench_args], env=env, check=True)
        # extra container bench file for valid_json baseline and llm_json comparator
        subprocess.run(['cargo', 'bench', '--bench', 'container_valid_bench', *bench_args], env=env, check=True)
        subprocess.run(['cargo', 'bench', '--bench', 'container_llm_bench', *bench_args], env=env, check=True)
        # streaming & writer benches
        subprocess.run(['cargo', 'bench', '--bench', 'stream_bench', *bench_args], env=env, check=True)
        subprocess.run(['cargo', 'bench', '--bench', 'writer_bench', *bench_args], env=env, check=True)
        print()
        print_success("Rust benchmarks completed")
        return True
    except subprocess.CalledProcessError as e:
        print_error(f"Rust benchmark failed with exit code {e.returncode}")
        return False
    except Exception as e:
        print_error(f"Rust benchmark failed: {e}")
        return False

def aggregate_results(config: Dict, output_file: str = 'docs/bench_table.md') -> bool:
    """Aggregate benchmark results."""
    print_header("Step 3/3: Aggregating results")

    # Pass environment variables to aggregate script
    env = os.environ.copy()
    env['JR_MIN_BYTES'] = str(config['jr_min_bytes'])
    env['JR_MEAS_SEC'] = str(config['jr_meas_sec'])
    env['JR_WARMUP_SEC'] = str(config['jr_warmup_sec'])
    env['JR_SAMPLE_SIZE'] = str(config['jr_sample_size'])
    env.setdefault('CARGO_TARGET_DIR', 'target_bench')

    try:
        # Ensure output directory exists
        Path(output_file).parent.mkdir(parents=True, exist_ok=True)
        with open(output_file, 'w', encoding='utf-8') as f:
            result = subprocess.run(
                [sys.executable, 'scripts/aggregate_bench.py'],
                stdout=f,
                stderr=subprocess.PIPE,
                text=True,
                env=env,
                check=True
            )
        print()
        print_success("Results aggregated")
        return True
    except subprocess.CalledProcessError as e:
        print_error(f"Aggregation failed with exit code {e.returncode}")
        if e.stderr:
            print(e.stderr, file=sys.stderr)
        return False
    except Exception as e:
        print_error(f"Aggregation failed: {e}")
        return False

def print_final_summary(start_time: float, profile_name: str):
    """Print final summary."""
    elapsed = time.time() - start_time
    minutes = int(elapsed // 60)
    seconds = int(elapsed % 60)
    
    print_header("✓ Benchmark completed successfully!")
    print_info(f"Profile: {profile_name}")
    print_info(f"Total time: {minutes}m {seconds}s")
    print()
    print_info("Results:")
    print_info("  - Python raw data:  python_bench.json")
    print_info("  - Comparison table: docs/bench_table.md")
    print()
    print(f"{Colors.BOLD}Tip:{Colors.ENDC} View results with:")
    if os.name == 'nt':
        print(f"  type docs\\bench_table.md")
    else:
        print(f"  cat docs/bench_table.md")
    print()

def main():
    parser = argparse.ArgumentParser(
        description='Run jsonrepair benchmarks with configurable profiles.',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Profiles:
  quick    - Fast iteration for development (~30s-1min)
  standard - Balanced accuracy and speed (~2-3min) [DEFAULT]
  heavy    - Maximum accuracy for official benchmarks (~5-8min)
  custom   - Use custom parameters (see --help for options)

Examples:
  python scripts/run_benchmarks.py quick
  python scripts/run_benchmarks.py standard
  python scripts/run_benchmarks.py heavy
        """
    )
    
    parser.add_argument('profile', nargs='?', default='standard',
                       choices=['quick', 'standard', 'heavy', 'custom'],
                       help='Benchmark profile to use (default: standard)')
    
    # Custom profile options
    parser.add_argument('--py-min-bytes', type=int, help='Python min bytes per iteration')
    parser.add_argument('--py-target-sec', type=float, help='Python target seconds per case')
    parser.add_argument('--py-warmup', type=int, help='Python warmup iterations')
    parser.add_argument('--jr-min-bytes', type=int, help='Rust min bytes per iteration')
    parser.add_argument('--jr-meas-sec', type=int, help='Rust measurement seconds')
    parser.add_argument('--jr-warmup-sec', type=int, help='Rust warmup seconds')
    parser.add_argument('--jr-sample-size', type=int, help='Rust sample size')
    
    args = parser.parse_args()
    
    # Print banner
    print(f"\n{Colors.BOLD}{Colors.HEADER}jsonrepair Benchmark Runner{Colors.ENDC}")
    print(f"{Colors.BOLD}Profile: {args.profile}{Colors.ENDC}")
    
    # Get configuration
    if args.profile == 'custom':
        config = PROFILES['standard'].copy()
        if args.py_min_bytes: config['py_min_bytes'] = args.py_min_bytes
        if args.py_target_sec: config['py_target_sec'] = args.py_target_sec
        if args.py_warmup: config['py_warmup'] = args.py_warmup
        if args.jr_min_bytes: config['jr_min_bytes'] = args.jr_min_bytes
        if args.jr_meas_sec: config['jr_meas_sec'] = args.jr_meas_sec
        if args.jr_warmup_sec: config['jr_warmup_sec'] = args.jr_warmup_sec
        if args.jr_sample_size: config['jr_sample_size'] = args.jr_sample_size
        config['description'] = 'Custom configuration'
        config['est_time'] = 'varies'
    else:
        config = PROFILES[args.profile]
    
    print(f"{Colors.BOLD}Description:{Colors.ENDC} {config['description']}")
    print(f"{Colors.BOLD}Estimated time:{Colors.ENDC} {config['est_time']}")
    print()
    
    # Print configuration
    print_info("Configuration:")
    print_info(f"  Python: min-bytes={config['py_min_bytes']}, "
              f"target-sec={config['py_target_sec']}, warmup={config['py_warmup']}")
    print_info(f"  Rust:   min-bytes={config['jr_min_bytes']}, "
              f"meas-sec={config['jr_meas_sec']}, "
              f"warmup-sec={config['jr_warmup_sec']}, "
              f"samples={config['jr_sample_size']}")
    
    # Check prerequisites
    if not check_python():
        sys.exit(1)
    if not check_cargo():
        sys.exit(1)
    
    start_time = time.time()
    
    # Run benchmarks
    if not run_python_benchmark(config):
        sys.exit(1)
    
    if not run_rust_benchmark(config):
        sys.exit(1)

    if not aggregate_results(config):
        sys.exit(1)
    
    # Print summary
    print_final_summary(start_time, args.profile)

if __name__ == '__main__':
    main()



