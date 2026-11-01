#!/usr/bin/env python3
"""Private paired runner. No network, credentials or Todos source in output."""
import argparse
import base64
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import subprocess
import tempfile
import zipfile


def canonical(unit, rows):
    if not isinstance(unit, str) or not unit:
        raise ValueError('unidade ausente')
    seen, result = {}, []
    for ident, number in rows:
        if not ident or not number:
            raise ValueError('identidade ausente')
        if number in seen and seen[number] != ident:
            raise ValueError('numero associado a IDs distintos')
        if number not in seen:
            seen[number] = ident
            result.append((ident, number))
    return dict(unidade=unit, processos=result)


def percentile(values, q):
    if not values or not 0 < q <= 1:
        raise ValueError('percentil sem amostras')
    return sorted(values)[math.ceil(q * len(values)) - 1]


def validate(sample, size):
    for key in ('total_ns', 'cpu_ns', 'mock_wait_ns', 'requests', 'response_bytes'):
        value = sample[key]
        if not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0:
            raise ValueError('metrica ausente ou fora do dominio')
    if sample['requests'] != 1 or sample['response_bytes'] != size:
        raise ValueError('orcamento HTTP excedido ou resposta diferente')
    if sample['mock_wait_ns'] > sample['total_ns']:
        raise ValueError('espera maior que operacao')


def load_cassette(card, archive):
    text = card.read_text()
    fields = dict(line.split(': ', 1) for line in text.split('---', 2)[1].splitlines()
                  if ': ' in line)
    raw_zip = archive.read_bytes()
    if hashlib.sha1(raw_zip).hexdigest() != fields['archive_sha1']:
        raise ValueError('SHA-1 do ZIP diferente')
    with zipfile.ZipFile(archive) as z:
        info = z.getinfo(fields['archive_member'])
        if info.file_size > 32_000_000:
            raise ValueError('cassette excessiva')
        raw = z.read(info)
    digest = hashlib.sha256(raw.replace(b'\r\n', b'\n')).hexdigest()
    if digest != fields['cassette_sha256_lf']:
        raise ValueError('SHA-256 da cassette diferente')
    data = json.loads(raw)
    if (data['schema_version'] != 1 or data['complete'] is not True
            or data['sei_root'].rstrip('/') != 'https://sei.treinamento.unb.br'
            or fields['installation'] != 'unb' or len(data['interactions']) != 1):
        raise ValueError('fora do escopo UnB de uma resposta')
    event = data['interactions'][0]
    request, response = event['request'], event['response']
    if (request['method'] != 'GET' or request['operation'] != 'procedimento_controlar'
            or response['status_code'] != 200):
        raise ValueError('fluxo diferente da primeira leitura')
    mime = {k.lower(): v for k, v in response['headers'].items()}['content-type']
    if 'text/html' not in mime.lower():
        raise ValueError('resposta nao HTML')
    body = base64.b64decode(response['body_base64'], validate=True)
    if not 0 < len(body) <= 8_000_000:
        raise ValueError('corpo fora do limite')
    return body, mime, digest


def run(command, job, env):
    completed = subprocess.run(command, env={**os.environ, **env, 'IES_PAIR_JOB': str(job)},
                               capture_output=True, text=True, timeout=180, check=False)
    if completed.returncode:
        # Do not echo a private HTML body or an upstream exception with a URL.
        raise RuntimeError('worker falhou; diagnostico retido, sem publicar dados da captura')
    rows = [s.removeprefix('PAIR_JSON=') for s in completed.stdout.splitlines()
            if s.startswith('PAIR_JSON=')]
    if len(rows) != 1:
        raise ValueError('worker sem resultado unico')
    return json.loads(rows[0])


def summarize(result, expected, size, repetitions):
    samples = result['samples']
    if len(samples) != repetitions or result['canonical'] != expected:
        raise ValueError('resultados ou quantidade de amostras diferentes')
    for sample in samples:
        validate(sample, size)
    rss = result['rss_peak_kib']
    if not isinstance(rss, int) or rss <= 0:
        raise ValueError('RSS ausente')
    report = dict(samples=repetitions, rss_peak_kib=rss, requests_per_operation=1,
                  response_body_bytes_per_operation=size, network='NOT_RUN')
    for key in ('total_ns', 'cpu_ns', 'mock_wait_ns', 'local_ns'):
        values = [(s['total_ns'] - s['mock_wait_ns']) if key == 'local_ns' else s[key]
                  for s in samples]
        report[key] = dict(p50=percentile(values, .5), p95=percentile(values, .95))
    count = len(expected['processos'])
    report['local_ns_per_process_p50'] = report['local_ns']['p50'] / count if count else None
    report['local_ns_per_mib_p50'] = report['local_ns']['p50'] * 1048576 / size
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--card', type=Path, required=True)
    parser.add_argument('--zip', type=Path, required=True)
    parser.add_argument('--ies-test-bin', required=True)
    parser.add_argument('--todos-python', required=True)
    parser.add_argument('--todos-src', type=Path, required=True)
    parser.add_argument('--todos-revision', required=True)
    parser.add_argument('--ies-revision', required=True)
    parser.add_argument('--unit', required=True)
    parser.add_argument('--repetitions', type=int, default=25)
    parser.add_argument('--max-local-p95-ms', type=float)
    parser.add_argument('--max-rss-mib', type=float)
    args = parser.parse_args()
    if not 5 <= args.repetitions <= 100:
        parser.error('repetitions deve estar entre 5 e 100')
    body, mime, digest = load_cassette(args.card, args.zip)
    results, expected = {}, None
    with tempfile.TemporaryDirectory(prefix='ies-paired-') as tmp:
        root = Path(tmp)
        (root / 'body').write_bytes(body)
        cookie = root / 'cookie'
        cookie.write_text('PHPSESSID=SINTETICO')
        cookie.chmod(0o600)
        for delay in (0, 10):
            job = root / 'job.json'
            job.write_text(json.dumps(dict(body=str(root / 'body'), mime=mime,
                cookie=str(cookie), unit=args.unit, repetitions=args.repetitions, delay_ms=delay)))
            workers = {
                'ies': ([args.ies_test_bin, '--ignored', '--exact',
                         'transporte::paired_worker::paired_unit_worker', '--nocapture'], {}),
                'todos': ([args.todos_python, str(Path(__file__).with_name('perf_pair_todos.py'))],
                          {'PYTHONPATH': str(args.todos_src.resolve())}),
            }
            # Reverse worker order for the second scenario.
            for name in (('ies', 'todos') if delay == 0 else ('todos', 'ies')):
                command, env = workers[name]
                value = run(command, job, env)
                if expected is None:
                    expected = value['canonical']
                if expected['unidade'] != args.unit:
                    raise ValueError('unidade diferente da solicitada')
                results[f'{name}_delay_{delay}ms'] = summarize(
                    value, expected, len(body), args.repetitions)
    report = dict(scope='mock transport + native box reader + native JSON; not full CLI',
                  cassette_sha256=digest, ies_revision=args.ies_revision,
                  todos_revision=args.todos_revision, platform=platform.platform(),
                  logical_cpus=os.cpu_count(), unique_processes=len(expected['processos']),
                  equivalence='PASS: unit and ordered unique (id, number)', results=results)
    violations = []
    for name, result in results.items():
        if (args.max_local_p95_ms is not None
                and result['local_ns']['p95'] > args.max_local_p95_ms * 1_000_000):
            violations.append(name + ': local p95')
        if args.max_rss_mib is not None and result['rss_peak_kib'] > args.max_rss_mib * 1024:
            violations.append(name + ': RSS')
    report['budget_violations'] = violations
    print(json.dumps(report, indent=2))
    if violations:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
