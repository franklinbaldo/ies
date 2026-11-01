---
type: Guide
title: Comparação pareada da primeira leitura da unidade
description: Contrato do ensaio offline sobre resposta verificada e projeção comum
---

# Escopo

Continuação de #95: resposta única da caixa resumida, sessão já disponível.
IES usa `read_box_via`, parser, validação da unidade e JSON nativos.
Todos usa `SEITransport` com `httpx.MockTransport`, `ler_caixa` e JSON nativos.
Este recorte é o núcleo transporte/leitura da primeira página, não a CLI
integral, login, troca de unidade, paginação ou API pública completa do Todos.
Não alterar os parsers nem suprimir suas validações para fazê-los concordar.

O runner privado recebe o ZIP e o card existentes; verifica SHA-1 do arquivo,
SHA-256 normalizado do membro, schema, conclusão, origem, método e operação.
Aceita apenas uma interação GET `procedimento_controlar`, HTTP 200 HTML.
Não extrai ZIP no filesystem nem usa URLs/cookies da captura para navegar.

## Equivalência e métricas

Antes de resumir tempos, cada amostra deve produzir a mesma unidade e a mesma
sequência única de pares (id, número), Recebidos antes de Gerados, mantendo a
primeira ocorrência por número como o Todos. Um número associado a IDs
distintos é erro, não deduplicação silenciosa. Campos adicionais permanecem
no processamento nativo, mas não entram na afirmação de equivalência.
Não confundir total de linhas com total de processos únicos.

Três aquecimentos descartados; cache de árvore Todos limpo entre operações,
mantido dentro da operação. Reuso do cliente mock; nenhuma sessão real.
Latência e CPU da operação incluem transporte, decoding, parser e JSON.
RSS é o VmHWM Linux do worker, incluindo imports/setup/aquecimento.
Espera mock é medida separadamente; local = total menos espera mock por
amostra. DNS, TLS, servidor e bytes no fio são NOT_RUN, nunca zero medido.
Bytes contam corpos de resposta consumidos; requests contam callbacks reais.
p50/p95 usam nearest-rank. Custo local por processo e por MiB é derivado
somente após equivalência. Não inferir preço monetário.

## Regressão

Orçamento determinístico: exatamente um GET e tamanho exato do corpo por
operação; resultados estáveis em todas as amostras. Rejeitar métricas
negativas, não finitas, amostras insuficientes e resultados diferentes.
Limites opcionais de p95 local e RSS devem ser calibrados no mesmo executor;
nunca adotar um fator de aceleração de outra máquina como gate de CI.
CI público testa apenas o runner e a costura Rust com dados sintéticos;
não baixa, instala ou publica código privado do Todos.

Treinamento live permanece separado: autorização por instalação, sessão
privada existente, uma leitura sem carga; sem sessão disponível, NOT_RUN.

## Execução privada

Instale o Todos autorizado e suas dependências fora do checkout público.
Compile `cargo test --release --locked --no-run --bin ies` e use o caminho
do executável de testes que o Cargo informa, não o binário da CLI.

```sh
python3 scripts/perf_pair.py \
  --card cassettes/unb/unidade-processos-paginado-unb--e02dbcfde8c4.md \
  --zip /caminho/privado/todos-sei-cassettes-unb.zip \
  --ies-test-bin /caminho/ies/target/release/deps/ies-HASH \
  --todos-python /caminho/privado/venv/bin/python \
  --todos-src /caminho/privado/todos/src \
  --todos-revision SHA_OU_DIGEST_DO_SNAPSHOT \
  --ies-revision SHA_IES --unit Unidade4
```

O runner emite apenas métricas agregadas e hashes. As identidades usadas
na conferência transitam localmente entre workers, sem aparecer no relatório.
`--max-local-p95-ms` e `--max-rss-mib` são limites opcionais para executor
controlado. O gate público canônico em `prek.toml` roda regressões sem Todos.

## Resultado do primeiro ensaio

Cassette original verificada, 103.392 bytes, 43 processos únicos. A comparação
inicial falhou no IES com `IdentificacoesDaUnidadeRepetidasNoControle`:
as duas barras da interface exibem a mesma unidade. A correção restrita ao
layout observado passou RED/GREEN e preservou recusas de contexto conflitante.
Nenhum byte da cassette foi alterado para obter equivalência.

Executor Linux x86-64, 9 CPUs lógicas visíveis, Rust 1.99.1 release,
Todos 0.24.254 (snapshot privado dos módulos utilizados), Python 3.15.0,
httpx 0.28.1, lxml 6.1.3, Pydantic 2.14.2. 25 amostras após 3 aquecimentos.
RSS inclui o worker e seus imports; não é memória incremental do parser.

| Espera mock | Implementação | Total p50/p95 ms | CPU p50/p95 ms | Local p50/p95 ms | RSS MiB |
| --- | --- | --- | --- | --- | --- |
| 0 ms | IES | 1,260 / 1,427 | 1,261 / 1,428 | 1,260 / 1,427 | 5,11 |
| 0 ms | Todos | 4,814 / 6,729 | 4,817 / 6,731 | 4,813 / 6,729 | 46,31 |
| 10 ms | IES | 11,531 / 11,844 | 1,460 / 1,779 | 1,430 / 1,756 | 5,18 |
| 10 ms | Todos | 15,803 / 16,646 | 5,738 / 6,584 | 5,601 / 6,473 | 46,47 |

Em todas as amostras: um GET e 103.392 bytes de corpo. Sem espera mock,
custo local mediano por processo: IES 29,30 µs, Todos 111,94 µs.
Por MiB de corpo: IES 12,78 ms, Todos 48,82 ms. São custos derivados de
uma única página, não curvas de escalabilidade. A igualdade abrange apenas
unidade e pares ID/NUP; Todos mantém campos e modelos adicionais.
O caso com espera demonstra somente sensibilidade à espera artificial.

Os [resultados agregados](paired-unit-results.json) preservam a medição e
os digests do snapshot privado e do binário de testes. Não publicar fontes
do Todos, corpos de resposta ou sessões junto ao relatório.

Decisão: preservar o orçamento de uma requisição; nenhum GET duplicado foi
observado. Não alterar o parser por intuição. A melhoria demonstrada foi
compatibilidade do contexto móvel/desktop, encontrada pelo gate funcional.
Rede real, TLS, login e sessão de treinamento: NOT_RUN. A indisponibilidade
do Jatobá impediu reutilizar uma sessão autorizada; não se criou novo login.
A issue #95 permanece aberta para operação integral e treinamento.
