---
type: SEIObservation
title: "UnB de treinamento: processo prazo definir e excluir, PASS após cinco correções"
description: "Ensaio de 10/11/2026 da #272: definir por dias e por data certa e excluir pela lista, todos com saída 0 e releitura independente depois de corrigir cinco diferenças da UnB"
---

# Processo prazo — UnB de treinamento, 10/11/2026

**Issue:** [#272](https://github.com/franklinbaldo/ies/issues/272).
**Origem:** somente `https://sei.treinamento.unb.br/sei` (UnB SEI 4.0.9 de
**treinamento**), conta pública `aluno1`, unidade `Unidade1`; sessão em
pasta `0700` com arquivos `0600`, fora do repositório.
**Processo:** `23106.001906/2026-14`, o mesmo do ensaio da #332.
**Execução:** binário release compilado localmente a partir de `main@f2aa9a4`
mais as correções desta PR; rede direta do sandbox.
**Autorização:** ensaio pedido pelo proprietário nesta sessão.

## Orçamento

Declarado **60**; revisado para **100** depois das tentativas de
diagnóstico. Usado: cerca de **95** requisições contadas pela CLI. Cada
`--confirmar` foi executado uma única vez, com trava `O_EXCL`; uma
confirmação que saiu com 3 nunca foi repetida sem correção de código entre
as tentativas, e as recusas com saída 1 não enviaram POST.

## Sequência

| # | Comando | Resultado |
| --- | --- | --- |
| 1 | `processo prazo NUP` (ler) | PASS `sem_prazo` |
| 2 | `--dias 10` (prévia) | PASS, sem escrita |
| 3 | `--dias 10 --confirmar` | **saída 3**; releitura independente: `19/11/2026` (gravou) |
| 4 | `--excluir` (prévia) | PASS |
| 5 | `--excluir --confirmar` | saída 1 antes do POST: ação de exclusão não reconhecida |
| 6 | `--excluir --confirmar` | saída 1 antes do POST: unidade vazia no formulário |
| 7 | `--excluir --confirmar` | **saída 3**; releitura independente: `sem_prazo` (gravou) |
| 8 | `--dias 5 --confirmar` | **saída 3**; releitura independente: `14/11/2026` (gravou) |
| 9 | `--excluir --confirmar` | **PASS saída 0**, `confirmado: true`, 9 requisições |
| 10 | `--ate 20/11/2026 --confirmar` | **PASS saída 0**, `20/11/2026`, 5 requisições |
| 11 | `--excluir --confirmar` | **PASS saída 0**, `sem_prazo`, 9 requisições |
| 12 | `--dias 5 --confirmar` | **PASS saída 0**, `14/11/2026`, 5 requisições |
| 13 | `--excluir --confirmar` | **PASS saída 0**, `sem_prazo`, 9 requisições |

## Diferenças da UnB corrigidas

Todas confirmadas no ZIP público da UnB (SHA-1
`21e0559015c56c403daf8933e5040504a7b83837`), sem repetir requisições.

1. **Dias viram data certa.** Pedido `--dias N`; a tela relida mostra a data
   calculada (`processo-prazo-unb`, interações 19→21). A releitura aceita a
   data a N dias corridos de hoje, com folga de um dia para o fuso.
2. **POST de definir responde 200** com o formulário redesenhado, não 302
   (interações 12 e 19). A prova fica com a releitura.
3. **URL de exclusão com parâmetros no meio**:
   `acao=controle_prazo_excluir&acao_origem=…&infra_hash=…`
   (`processo-prazo-excluir-unb`, interação 13).
4. **Unidade da lista**: o formulário recebia unidade vazia, mas a lista
   publica `lnkInfraUnidade`; passa a unidade lida na abertura.
5. **Link do prazo envelhece**: com prazo, a barra leva
   `id_controle_prazo` (interação 11), que some depois da exclusão
   (interação 19). A testemunha reabre o processo e usa o link novo.

## Resultado e resíduo

**PASS** para definir por dias, definir por data certa e excluir. Pela
decisão do proprietário de 10/11/2026, a escrita de prazo fica aprovada
para produção (`src/origem_de_treinamento.rs`).
Resíduo: nenhum; o processo terminou `sem_prazo`.
