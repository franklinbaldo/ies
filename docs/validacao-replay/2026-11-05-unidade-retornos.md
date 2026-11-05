---
type: SEIObservation
title: Replay de retornos programados nas sete instalacoes de treinamento
description: 14 paginas mensais conferidas com as cassettes originais, SHA-1 e SHA-256, sem rede SEI
---

# Replay do acervo — `unidade retornos` — 05/11/2026

**PASS** — [workflow do replay](https://github.com/franklinbaldo/ies/actions/runs/42096471505).
O teste Rust `unidade_retornos::acervo::replay_retornos_no_acervo`
executou em `scripts/replay` com seis ZIPs originais, cobrindo os sete
cards `unidade-listar-retornos-*`. Foram conferidos SHA-1 dos ZIPs,
SHA-256 de cada membro JSON, `complete`, schema e origem.

| Instalação | Meses completos no acervo | Retornos nas duas leituras | Recusas |
| --- | ---: | --- | ---: |
| UnB | 2 | 2 e 2 | 0 |
| UFRJ | 2 | 0 e 1 | 0 |
| Goiás | 2 | 0 e 1 | 0 |
| PEN1 | 2 | 0 e 1 | 0 |
| PEN2 | 2 | 0 e 1 | 0 |
| PEN3 | 2 | 0 e 2 | 0 |
| PEN4 | 2 | 0 e 1 | 0 |
| **Total** | **14** | **14/14 lidas** | **0** |

**RED observado no replay antes da correção:** [execução anterior](https://github.com/franklinbaldo/ies/actions/runs/42095213665):
7 leituras aceitas e 7 recusadas como `lista_de_retornos_incompleta`.
A inspeção estrutural, sem imprimir URLs nem HTML, identificou que a
listagem completa de um mês pode publicar somente `aguardando` dentro
de `divAguardandoAreaTabela`, sem uma tabela `devolver`. Esta forma
foi observada nas sete instalações. O leitor passou a aceitar a forma
específica, sem aceitar uma tabela isolada sem a moldura mensal.
Os testes sintéticos preservam essa recusa e os orçamentos de GET.

**Limite:** o replay não envia requisições à instalação (0 GET/POST
ao vivo), não demonstra login, sessão atual, paginação independente
das duas tabelas ou procura de `diaHoje` em mês vizinho. Isso requer
evidência adicional. Nenhuma escrita foi realizada no SEI.
