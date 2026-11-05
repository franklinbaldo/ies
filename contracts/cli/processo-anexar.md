---
type: CommandContract
title: Prévia de anexação de processo ao principal
operation: processo.anexar
command: ies processo anexar NUP --a NUP_PRINCIPAL [--confirmar]
effect: write-process-state
implementation: partial
---

# Anexar processo (issue #270)

`ies processo anexar NUP --a NUP_PRINCIPAL [--confirmar]` lê pela interface Web a anexação do processo NUP ao processo principal, sem REST ou SOAP. A anexação é potencialmente irreversível para o usuário comum.

## Estado desta fatia

**Prévia parcial.** Lê a árvore do principal e o link assinado `procedimento_anexar`, GET de `frmProcedimentoAnexar`, consulta AJAX de identificação `protocolo_RI1023` e consulta de assinaturas `procedimento_anexacao_verificar`, cada uma **uma única vez**. Sem `--confirmar`, não envia o POST de anexação. `--confirmar` recusa **antes de acessar a sessão ou rede**, com `confirmacao_de_anexacao_nao_validada`, até existirem testes de transporte, resultado 302 e verificação por leitura.

O processo principal é conferido por NUP na abertura, e o anexado pelo NUP completo retornado pela consulta. O destino igual ao principal, duplicado na lista ou identificado ambiguamente é recusado. A lista pode vir ausente, como na tela de zero anexados observada. `N` na consulta de assinaturas significa que há documento não assinado; a prévia **avisa**, mas não executa.

## Orçamento de requests

A abertura do principal reutiliza o índice/fluxo normal e pode precisar de vários GETs, que são contados. A partir da árvore: um GET da tela de anexação e **dois POSTs de leitura** (`protocolo_RI1023` e `procedimento_anexacao_verificar`). O comando retorna a contagem observada `requisicoes`; não há retry nem 302 seguido. A confirmação atual: **zero requisições** e saída 1. Orçamento da escrita futura só será contratado depois de observar e testar o formulário completo e a verificação; nunca presumir sucesso pelo 302.

## Recusas próprias

`anexacao_ao_mesmo_processo`, `processo_ja_anexado`, `anexacao_nao_publicada_na_barra`, `links_de_anexar_repetidos_na_barra`, `endereco_de_anexar_nao_reconhecido`, `endereco_de_anexar_de_outro_processo`, `formulario_de_anexacao_ausente_ou_repetido`, `controles_da_anexacao_ausentes_ou_repetidos`, `listas_de_anexados_repetidas`, `ids_de_anexados_repetidos`, `id_do_anexado_ausente_ou_invalido`, `processo_a_anexar_nao_identificado`, `identidade_de_anexado_ambigua`, `anexacao_nao_verificada`, `nup_do_anexado_diferente_do_solicitado`, `confirmacao_de_anexacao_nao_validada`. O transportador e o leitor da árvore conservam seus próprios subcódigos tipados.

## Subcódigos

| Subcódigo | Fato identificado |
| --- | --- |
| `anexacao_nao_publicada_na_barra` | O processo aberto não publica ação de anexar. |
| `confirmacao_de_anexacao_nao_validada` | Escrita ainda bloqueada antes da sessão. |
| `endereco_de_anexar_de_outro_processo` | Link assinado aponta para outro processo. |
| `endereco_de_anexar_nao_reconhecido` | Link assinado da ação inválido. |
| `links_de_anexar_repetidos_na_barra` | Mais de um link de anexar na barra. |
| `tela_de_anexacao_redirecionada` | GET da tela retornou redirecionamento não seguido. |
| `anexacao_ao_mesmo_processo` | Processo anexado e principal possuem a mesma identidade. |
| `processo_ja_anexado` | Identificador já consta na lista observada. |
| `formulario_de_anexacao_ausente_ou_repetido` | Formulário ausente ou duplicado. |
| `controles_da_anexacao_ausentes_ou_repetidos` | Campos obrigatórios ausentes, duplicados ou com tipo indevido. |
| `listas_de_anexados_repetidas` | Duas tabelas da lista observadas. |
| `ids_de_anexados_repetidos` | Mesmo id aparece mais de uma vez na lista. |
| `processo_a_anexar_nao_identificado` | Consulta não publicou identidade válida. |
| `identidade_de_anexado_ambigua` | Complementos de identidade repetidos. |
| `anexacao_nao_verificada` | Consulta de assinatura não publicou S/N únicos. |

## Critérios ainda pendentes

Formulário de escrita derivado da tela, envio único sem retry, aceite 302 conferido, verificação por leitura nova da árvore do principal, efeito incerto com saída 3, replay de cassettes e prévia/live de UnB e segunda instalação de treinamento. Nenhuma escrita é autorizada fora de treinamento.
