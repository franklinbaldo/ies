---
type: SEIObservation
title: "Cobertura das sete instalações de treinamento: relacionar, andamento, incluir externo e enviar"
description: "Ensaio de 10/11/2026 em PEN1, PEN2, PEN3, PEN4, UFRJ e GO, complementando a UnB: as quatro escritas passaram com saída 0 e releitura independente em todas, sem alteração de código"
---

# Cobertura das sete instalações — 10/11/2026

**Regra:** aprovação para produção exige ensaio em todas as instalações de
treinamento (decisão do proprietário de 10/11/2026; `src/origem_de_treinamento.rs`).
A UnB foi ensaiada antes:
[incluir externo](2026-11-09-documento-incluir-externo-unb.md),
[relacionar e andamento](2026-11-10-processo-relacionar-andamento-unb.md),
[enviar](2026-11-10-processo-enviar-remover-anotacoes-unb.md). O prazo tem o
[relatório próprio das sete](2026-11-10-processo-prazo-7-instalacoes.md).
**Contas:** as públicas da [tabela autorizada](../interfaces/credenciais-de-treinamento.md);
sessões em pastas `0700` com arquivos `0600`, fora do repositório.
**Execução:** binário release de `main@c114447`, **sem alteração de código**;
rede direta do sandbox. Cada `--confirmar` foi executado uma única vez, com
trava `O_EXCL`; nenhum saiu com 3.

## Processos

| Instalação | Unidade | Processo A | Processo B |
| --- | --- | --- | --- |
| PEN1 5.0.4 | TESTE | `043491/2026-21` | `043490/2026-86` |
| PEN2 | TESTE | `98290.000142/2026-09` | `98290.000139/2026-87` |
| PEN3 5.0.4 | TESTE | `98390.000088/2026-65` | `98390.000089/2026-18` |
| PEN4 5.0.4 | TESTE | `98490.000104/2026-91` | `98490.000107/2026-25` |
| UFRJ | ADCENTRAL | `99079.901797/2026-65` | `99079.901792/2026-32` |
| GO 5.0.3 | SEAD/CDE-09516 | `202600005004933` | `202600005006023` |

## Relacionar (A com B, depois desfazer)

Orçamento declarado **160**; usado cerca de **126**. Nas seis: prévia 4
requisições, `--confirmar` **saída 0** (6), nova prévia `sem_efeito: true`,
`--remover --confirmar` **saída 0** (5), nova prévia `sem_efeito: false`.
Resíduo: nenhum.

## Andamento (processo A)

Orçamento declarado **100**; usado cerca de **96**. Nas seis: prévia 4
requisições, `--confirmar` **saída 0** (8), histórico com um item a mais na
releitura independente. Resíduo: o andamento "Ensaio IES #259: andamento de
teste em treinamento" no histórico de cada processo A.

## Incluir externo (processo A)

Orçamento declarado **120**; usado cerca de **113**. Prévia com 6
requisições (formulário `escolher(-1)` do SEI 5) e `--confirmar` com
**saída 0** (9), `reconciliado_por_leitura: true`. Releitura independente por
`ies documento`: ícone PDF e **SHA-256 idêntico** ao enviado, em todas.

| Instalação | Tipo | Documento |
| --- | --- | --- |
| PEN1 | Anexo (263) | `0012369` |
| PEN2 | Anexo (263) | `0002163` |
| PEN3 | Anexo (263) | `0001067` |
| PEN4 | Anexo (263) | `0000908` |
| UFRJ | Anexo (293) | `0003129` |
| GO | Modelo para teste de numeração (4863) | `8446442` |

O GO não oferece o tipo "Anexo" (prévia: `tipo_externo_nao_oferecido`, sem
upload). O tipo de teste foi escolhido na lista da cassette pública
`cadastro-listas-go` (SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`).
Resíduo: um documento externo de teste por instalação, na tabela acima.

## Enviar com `--manter-aberto --remover-anotacoes` (processo B)

Orçamento declarado **110**; usado cerca de **102**. Destinos tirados das
cassettes públicas de tramitação: `TESTE_1_1` (PEN), `PR-A` (UFRJ), `TESTE`
(GO). Nas seis: prévia 5 requisições, `metodo_remocao: checkbox_no_envio`,
`anotacao_antes: false`; `--confirmar` **saída 0** (8),
`anotacao_depois: false`; a releitura independente mostra o processo aberto na
unidade de origem e no destino. **Não comprovado:** a remoção de anotação
preexistente, porque nenhum processo tinha anotação. Resíduo: o processo B
ficou aberto também na unidade de destino.

## Resultado

**PASS nas sete instalações** para `processo relacionar`, `processo
andamento`, `documento incluir-externo` e `processo enviar
--remover-anotacoes`. Com o prazo, as cinco escritas cumprem a regra de
aprovação.
