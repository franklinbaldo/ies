"""Adapter for an authorized private Todos installation; no copied source."""
import asyncio
import json
import os
from pathlib import Path
import time

import httpx
from todos.sei.caixa import ler_caixa
from todos.sei.lxml_html import arvore
from todos.sei.transport import SEITransport

from perf_pair import canonical


async def main():
    job = json.loads(Path(os.environ['IES_PAIR_JOB']).read_text())
    body = Path(job['body']).read_bytes()
    counts = dict(requests=0, response_bytes=0, mock_wait_ns=0)

    async def respond(request):
        if request.method != 'GET':
            raise ValueError('metodo fora do contrato')
        counts['requests'] += 1
        start = time.perf_counter_ns()
        if job['delay_ms']:
            await asyncio.sleep(job['delay_ms'] / 1000)
        counts['mock_wait_ns'] += time.perf_counter_ns() - start
        counts['response_bytes'] += len(body)
        return httpx.Response(200, content=body, headers={'content-type': job['mime']})

    samples, expected = [], None
    async with httpx.AsyncClient(transport=httpx.MockTransport(respond)) as client:
        transport = SEITransport('https://sei.treinamento.unb.br/sei/', client=client)
        for index in range(job['repetitions'] + 3):
            arvore.cache_clear()
            counts.update(requests=0, response_bytes=0, mock_wait_ns=0)
            start, cpu = time.perf_counter_ns(), time.process_time_ns()
            response = await transport.get('controlador.php?acao=procedimento_controlar')
            box = ler_caixa([transport.text(response)])
            serialized = box.model_dump_json()
            sample = dict(total_ns=time.perf_counter_ns() - start,
                          cpu_ns=time.process_time_ns() - cpu, **counts)
            if not serialized:
                raise ValueError('serializacao vazia')
            value = canonical(box.unidade, [(r['key'], r['Processo']) for r in box.linhas])
            if value['unidade'] != job['unit'] or (expected is not None and expected != value):
                raise ValueError('resultado mudou')
            expected = value
            if index >= 3:
                samples.append(sample)
        if transport.requisicoes != job['repetitions'] + 3:
            raise ValueError('contagem do transporte nao confere')
    status = Path('/proc/self/status').read_text().splitlines()
    rss = int(next(s for s in status if s.startswith('VmHWM:')).split()[1])
    print('PAIR_JSON=' + json.dumps(dict(canonical=expected, samples=samples, rss_peak_kib=rss)))


if __name__ == '__main__':
    asyncio.run(main())
