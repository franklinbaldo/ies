---
type: CommandContract
title: Sobrestar e retomar processo na unidade da sessão
operation: processo.sobrestar
command: ies processo sobrestar NUP --motivo TEXTO [--confirmar]
effect: write-process-state
implementation: partial
---

# Sobrestar e retomar — #269

Comandos: `ies processo sobrestar NUP --motivo TEXTO [--confirmar]` e
`ies processo retomar NUP [--confirmar]`. Só Web, com a unidade da sessão
publicada pelo SEI. Nenhuma URL assinada, senha ou cookie é exibido.

## Entrada e prévia efetivamente implementada

NUP usa o mesmo validador de `ies processo NUP`. `--motivo` é
obrigatório em sobrestar; branco ou vazio recusa com código de saída 2
(`motivo_do_sobrestamento_vazio`) antes de qualquer arquivo ou rede.

Sem `--confirmar`: lê o processo pela mesma abertura de
[`processo concluir`](processo-concluir.md), sem link guardado da árvore,
pois ele não prova a unidade. Confere `lnkInfraUnidade` na tela de trabalho
e `aberto_em` do nó raiz. Sobrestar exige unidade ativa dentre as unidades
abertas e um único link assinado de `procedimento_sobrestar` na barra
do processo. Retomar exige `AbertoEm::Sobrestado` na unidade ativa
e um único `onclick="removerSobrestamentoProcesso();"` com URL da
tela do nó da mesma árvore. Confere a assinatura/ID, sem seguir 302
e sem executar JavaScript.

Resposta: `nup`, `id_procedimento`, `unidade`, `acao`, `motivo`
(quando sobrestar), `origem: web`, `confirmado: false`,
`alterado: false`, `requisicoes`. Não publica URL nem hash.

**Estado parcial:** `--confirmar` é recusado com
`confirmacao_de_sobrestamento_nao_implementada`, antes de ler
a sessão e sem rede. NÃO existe escrita remota nesta fatia. A
implementação posterior exigirá prévia, uma única escrita derivada
da tela, zero retries, e leitura posterior para reconciliar o efeito
(saída 3 se incerto). Testes live apenas em treinamento.

## Requisições e evidências

| Caso | Requisições | Escritas |
| --- | --- | --- |
| Uso inválido ou confirmação bloqueada | 0 | 0 |
| Prévia a frio pelo Controle | 3 GET | 0 |
| Prévia com link de trabalho guardado | 2 GET | 0 |
| Recusa de situação/barra | só abertura | 0 |

Cenários derivados de [Todos](https://github.com/franklinbaldo/todos/blob/main/src/todos/ciclo.py),
com cassettes de [UnB](../../cassettes/unb/processo-sobrestar-retomar-unb--cde2a847ebd8.md)
e [GO](../../cassettes/go/processo-sobrestar-retomar-go--4fb6c8fe18df.md).
RED no CI inicial: **5 testes falhando**. Replay e live: NOT_RUN, 0 requisições.

## Recusas tipadas (contrato de subcódigos)

| Subcódigo | Condição |
| --- | --- |
| `motivo_do_sobrestamento_vazio` | Não há motivo válido |
| `unidade_do_sobrestamento_nao_publicada` | Unidade não observada |
| `situacao_antes_do_sobrestamento_nao_lida` | Situação não observada |
| `barra_do_sobrestamento_nao_interpretada` | Barra fora da forma |
| `acoes_de_sobrestamento_repetidas_na_barra` | Ações duplicadas |
| `endereco_do_sobrestamento_nao_reconhecido` | URL inválida |
| `endereco_do_sobrestamento_de_outro_processo` | ID diferente |
| `tela_do_no_da_retomada_nao_reconhecida` | URL da raiz não reconhecida |
| `processo_ja_sobrestado` | Já sobrestado |
| `processo_nao_sobrestado` | Retomada não aplicável |
| `processo_nao_aberto_na_unidade_para_sobrestar` | Outra unidade |
| `processo_sobrestado_em_outra_unidade` | Outra unidade |
| `acao_de_sobrestamento_nao_oferecida` | Barra não oferece |
| `confirmacao_de_sobrestamento_nao_implementada` | Escrita recusada |

[Decisão](../../docs/decisoes/processo-sobrestar.md) ·
[live](../../docs/validacao-live/2026-11-05-processo-sobrestar.md) ·
[PR #279](https://github.com/franklinbaldo/ies/pull/279).
