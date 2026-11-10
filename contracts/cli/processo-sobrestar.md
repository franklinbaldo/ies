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

## Entrada, prévia e confirmação implementadas offline

NUP usa o mesmo validador de `ies processo NUP`. `--motivo` é
obrigatório em sobrestar; branco ou vazio recusa com código de saída 2
(`motivo_do_sobrestamento_vazio`) antes de qualquer arquivo ou rede.
O motivo aceito é preservado integralmente no formulário e na saída JSON,
incluindo espaços nas extremidades e quebras de linha.

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

**Confirmação implementada no código:** `--confirmar` realiza a mesma
abertura da prévia, seguida de um GET da tela de sobrestar ou do nó,
de uma única escrita (POST com `rdoSobrestar=S` e `txaMotivo`,
ou GET da função de retirada) e uma nova leitura da árvore. Não há
retry de escrita, execução de JavaScript ou seguimento do 302. A
conferência exige `Sobrestado` na unidade para sobrestar, e
`AbertoEm::Unidades` com a unidade ativa para retomar. Se a
verificação falhar, sai com código 1 (escrita não confirmada). Um 302
isolado nunca prova a escrita. O JSON de sucesso só publica
`alterado:true` e `reconciliado_por_leitura:true` após
verificação. **Autenticação e escrita live ainda NOT_RUN**; o
código não foi exercitado em instalação de treinamento nesta PR.

## Requisições e evidências

| Caso | Requisições | Escritas |
| --- | --- | --- |
| Uso inválido (NUP/motivo) | 0 | 0 |
| Prévia a frio pelo Controle | 3 GET | 0 |
| Prévia com link de trabalho guardado | 2 GET | 0 |
| Confirmação sobrestar a frio | 5 GET + 1 POST = 6 | 1 POST |
| Confirmação retomar a frio | 6 GET = 6 | 1 GET de escrita |
| Recusa de situação/barra | só abertura | 0 |
| Recusa de tela | abertura + 1 GET | 0 |
| Falha após escrita | mesma contagem + leitura verificadora | 1, sem retry |

Cenários derivados de [Todos](https://github.com/franklinbaldo/todos/blob/main/src/todos/ciclo.py),
com cassettes de [UnB](../../cassettes/unb/processo-sobrestar-retomar-unb--cde2a847ebd8.md)
e [GO](../../cassettes/go/processo-sobrestar-retomar-go--4fb6c8fe18df.md).
RED da etapa inicial #279: 5 testes falhando. **RED da presente PR #284:
4 testes novos falhando**, antes do código (CI 42104973101).
As cinco instalações adicionais e o replay das cassettes do acervo
ainda não foram testados com esta implementação. Live: NOT_RUN,
0 requisições ao SEI e nenhum resíduo.

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
| `tela_de_sobrestamento_redirecionada` | GET da tela devolveu 302, antes do POST |
| `charset_da_tela_de_sobrestar_nao_suportado` | Charset da tela não conhecido |
| `tela_de_sobrestar_nao_interpretada` | HTML da janela não interpretado |
| `formulario_de_sobrestar_ausente` | Formulário ausente |
| `formulario_de_sobrestar_de_outro_processo` | Identidade da janela diferente |
| `opcao_somente_sobrestar_nao_oferecida` | Rádio S ausente, repetido ou marcado de saída |
| `tela_de_sobrestar_com_destino_preenchido` | Vínculo com outro processo preenchido |
| `campo_inesperado_na_tela_de_sobrestar` | Campo extra, ausente, desabilitado ou repetido |
| `endereco_de_retomar_ausente_na_tela` | Função/variável não publicadas |
| `endereco_de_retomar_fora_da_forma` | Forma da variável ou função não observada |
| `enderecos_de_retomar_repetidos_na_tela` | Duas URLs de escrita |
| `endereco_de_retomar_nao_reconhecido` | URL assinada não reconhecida |
| `endereco_de_retomar_de_outro_processo` | URL de outro id |
| `sobrestamento_ou_retomada_nao_aplicados` | Leitura confirma que o estado não mudou |
| `nao_confirmado_no_sobrestamento` | Leitura posterior falhou ou não decidiu (saída 1) |

[Decisão](../../docs/decisoes/processo-sobrestar.md) ·
[live](../../docs/validacao-live/2026-11-05-processo-sobrestar.md) ·
[PR #284](https://github.com/franklinbaldo/ies/pull/284) (continuação da #279).
