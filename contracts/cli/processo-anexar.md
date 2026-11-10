---
type: CommandContract
title: Prévia e confirmação de anexação de processo ao principal
operation: processo.anexar
command: ies processo anexar NUP --a NUP_PRINCIPAL [--confirmar]
effect: write-process-state
implementation: partial
---

# Anexar processo (issue #270)

`ies processo anexar NUP --a NUP_PRINCIPAL [--confirmar]` lê pela interface Web a anexação do processo NUP ao processo principal, sem REST ou SOAP. A anexação é potencialmente irreversível para o usuário comum.

## Estado desta fatia

**Prévia e confirmação com prova sintética.** A CLI lê a árvore do principal e o link assinado `procedimento_anexar`, faz GET de `frmProcedimentoAnexar` e as consultas AJAX `protocolo_RI1023` e `procedimento_anexacao_verificar`. Sem `--confirmar`, não envia o POST de anexação. Com `--confirmar`, somente em origens de treinamento autorizadas, deriva os campos do formulário servido, efetua **um único POST** e exige aceite do redirecionamento, nova leitura da lista e árvore do principal que contenha o anexado. Quando o efeito não pode ser comprovado, informa **saída 1 e não repete a escrita**. Este caminho passou em laboratório com CLI Rust real ↔ anti-IES ↔ SQLite, mas ainda não demonstra fidelidade semântica às cassettes nem validação externa.

O processo principal é conferido por NUP na abertura, e o anexado pelo NUP completo retornado pela consulta. O destino igual ao principal, duplicado na lista ou identificado ambiguamente é recusado. A lista pode vir ausente, como na tela de zero anexados observada. `N` na consulta de assinaturas significa que há documento não assinado; a prévia **avisa**, mas não executa.

## Orçamento de requests

A abertura do principal reutiliza o índice/fluxo normal e pode precisar de vários GETs, que são contados. A partir da árvore: um GET da tela de anexação e **dois POSTs de leitura** (`protocolo_RI1023` e `procedimento_anexacao_verificar`). A confirmação acrescenta **um POST mutador**, um GET da lista de anexados e a releitura do processo principal; os testes sintéticos verificam POST único e ausência de retry mesmo sob resposta HTTP 503 injetada. O comando informa a contagem observada em `requisicoes` quando retorna normalmente; o aceite HTTP isolado não prova o efeito.

## Recusas próprias

`anexacao_ao_mesmo_processo`, `processo_ja_anexado`, `FalhaAnexar::LinkNaoPublicado`, `links_de_anexar_repetidos_na_barra`, `endereco_de_anexar_nao_reconhecido`, `ErroAnexarNaBarra::EnderecoDeOutroProcesso`, `formulario_de_anexacao_ausente_ou_repetido`, `controles_da_anexacao_ausentes_ou_repetidos`, `listas_de_anexados_repetidas`, `ids_de_anexados_repetidos`, `id_do_anexado_ausente_ou_invalido`, `processo_a_anexar_nao_identificado`, `identidade_de_anexado_ambigua`, `anexacao_nao_verificada`, `nup_do_anexado_diferente_do_solicitado`, `FalhaAnexar::NaoConfirmado` (legado da fatia de prévia; não é a recusa atual do caminho confirmado). O transportador e o leitor da árvore conservam seus próprios subcódigos tipados.

## Subcódigos

| Variante ou `onde` (stderr) | Fato identificado |
| --- | --- |
| `FalhaAnexar::LinkNaoPublicado` | O processo aberto não publica ação de anexar. |
| `FalhaAnexar::NaoConfirmado` | Recusa histórica da fatia de prévia; não representa o caminho atual de confirmação em treinamento. |
| `ErroAnexarNaBarra::EnderecoDeOutroProcesso` | Link assinado aponta para outro processo. |
| `endereco_de_anexar_nao_reconhecido` | Link assinado da ação inválido. |
| `links_de_anexar_repetidos_na_barra` | Mais de um link de anexar na barra. |
| `FalhaAnexar::TelaRedirecionada` | GET da tela retornou redirecionamento não seguido. |
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

**Comprovado apenas sinteticamente:** formulário derivado, POST único, aceite 302 e releitura por outra execução da CLI sobre SQLite, além de saída 1 para HTTP 503 injetado sem retry. **Pendentes:** confronto semântico independente IES e anti-IES ↔ cassettes e prévia/validação externa em UnB e segunda instalação de treinamento, com relatório e resíduo. Nenhuma escrita é autorizada fora de treinamento.
