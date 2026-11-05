---
type: Contract
title: Subcódigos de árvore do processo
description: Inventário literal complementar de falhas por contexto emissor
---

# Subcódigos de árvore do processo

Este complemento enumera as condições de árvore, pastas, anexações e documentos movidos, compartilhados pela leitura de processos e documentos.
As tabelas completam os contratos das operações; não mudam códigos de
saída, orçamento nem regras de recuperação. Delegações preservam o
subcódigo do leitor chamado. O mesmo literal pode aparecer em `subcodigo`
ou em uma falha aninhada; interpretar junto à operação e à etapa.
Ver [regras e contextos compartilhados](../../docs/erros-tipados.md).

Contratos de operação: [Processo](processo-obter-arvore.md), [pastas](processo-obter-pastas.md), [movidos](processo-obter-movidos.md) e [documentos](documento-obter-web.md).

## Falhas tipadas

### arvore_processo/conteudo

Emissor: [src/arvore_processo/conteudo.rs](../../src/arvore_processo/conteudo.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `endereco_do_conteudo_de_outro_documento` | Endereço do conteúdo de outro documento. |
| `endereco_do_conteudo_nao_reconhecido` | Endereço do conteúdo não reconhecido. |
| `link_do_conteudo_ausente_na_arvore` | Link do conteúdo ausente na árvore. |
| `link_do_conteudo_fora_da_forma_observada` | Link do conteúdo fora da forma observada. |
| `link_do_conteudo_repetido_na_arvore` | Link do conteúdo repetido na árvore. |

### arvore_processo/erro

Emissor: [src/arvore_processo/erro.rs](../../src/arvore_processo/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `acao_de_abrir_pastas_fora_da_forma_observada` | Ação de abrir pastas fora da forma observada. |
| `acoes_de_abrir_pastas_repetidas` | Ações de abrir pastas repetidas. |
| `aguarde_com_pai_diferente_de_pasta` | Aguarde com pai diferente de pasta. |
| `aguarde_fora_da_forma_observada` | Aguarde fora da forma observada. |
| `argumento_de_no_com_tipo_diferente_do_observado` | Argumento de no com tipo diferente do observado. |
| `argumento_de_no_nao_literal` | Argumento de no não literal. |
| `arvore_nao_reconhecida` | Árvore não reconhecida. |
| `barra_de_no_fora_da_forma_observada` | Barra de no fora da forma observada. |
| `contagem_da_pasta_diferente_da_declarada` | Contagem da pasta diferente da declarada. |
| `declaracao_de_no_fora_da_forma_observada` | Declaração de no fora da forma observada. |
| `destino_do_documento_movido_fora_da_forma_observada` | Destino do documento movido fora da forma observada. |
| `destinos_do_documento_movido_repetidos` | Destinos do documento movido repetidos. |
| `documento_com_pai_diferente_do_processo` | Documento com pai diferente do processo. |
| `documento_da_pasta_nao_declarado` | Documento da pasta não declarado. |
| `documento_da_pasta_sem_identidade_na_barra` | Documento da pasta sem identidade na barra. |
| `documento_movido_com_pai_diferente_do_processo` | Documento movido com pai diferente do processo. |
| `documento_movido_fora_da_forma_observada` | Documento movido fora da forma observada. |
| `documento_movido_repetido_na_arvore` | Documento movido repetido na árvore. |
| `documento_repetido_na_arvore` | Documento repetido na árvore. |
| `endereco_de_abrir_pastas_nao_reconhecido` | Endereço de abrir pastas não reconhecido. |
| `escape_de_no_nao_reconhecido` | Escape de no não reconhecido. |
| `html_do_no_do_processo_fora_da_forma_observada` | HTML do no do processo fora da forma observada. |
| `id_de_no_nao_decimal` | ID de no não decimal. |
| `id_do_processo_na_arvore_diferente_do_aberto` | ID do processo na árvore diferente do aberto. |
| `identidade_do_processo_diferente_da_pedida` | Identidade do processo diferente da pedida. |
| `identidades_divergentes_na_barra_do_documento` | Identidades divergentes na barra do documento. |
| `indice_de_no_fora_da_sequencia` | Índice de no fora da sequência. |
| `no_com_pai_no_processo_anexado` | No com pai no processo anexado. |
| `no_do_processo_ausente_na_arvore` | No do processo ausente na árvore. |
| `no_do_processo_com_pai` | No do processo com pai. |
| `nos_do_processo_repetidos_na_arvore` | Nos do processo repetidos na árvore. |
| `nup_do_no_do_processo_anexado_inconsistente` | NUP do no do processo anexado inconsistente. |
| `nup_do_no_do_processo_inconsistente` | NUP do no do processo inconsistente. |
| `pasta_carregada_pela_metade` | Pasta carregada pela metade. |
| `pasta_com_pai_diferente_do_processo` | Pasta com pai diferente do processo. |
| `pasta_fora_da_forma_observada` | Pasta fora da forma observada. |
| `pasta_repetida_na_arvore` | Pasta repetida na árvore. |
| `pasta_sem_protocolos_declarados` | Pasta sem protocolos declarados. |
| `pastas_nao_carregadas_apos_abrir_pastas` | Pastas não carregadas após abrir pastas. |
| `pastas_nao_carregadas_sem_acao_de_abrir` | Pastas não carregadas sem ação de abrir. |
| `processo_anexado_com_pai_diferente_do_processo` | Processo anexado com pai diferente do processo. |
| `processo_anexado_fora_da_forma_observada` | Processo anexado fora da forma observada. |
| `processo_anexado_igual_ao_processo` | Processo anexado igual ao processo. |
| `processo_anexado_repetido_na_arvore` | Processo anexado repetido na árvore. |
| `protocolo_declarado_ausente_da_pasta` | Protocolo declarado ausente da pasta. |
| `protocolo_repetido_na_pasta` | Protocolo repetido na pasta. |
| `protocolos_da_pasta_declarados_mais_de_uma_vez` | Protocolos da pasta declarados mais de uma vez. |
| `protocolos_da_pasta_fora_da_forma_observada` | Protocolos da pasta fora da forma observada. |
| `protocolos_declarados_para_pasta_ausente` | Protocolos declarados para pasta ausente. |
| `quantidade_de_argumentos_do_no_diferente_da_observada` | Quantidade de argumentos do no diferente da observada. |
| `src_de_no_fora_da_forma_observada` | Src de no fora da forma observada. |
| `texto_de_no_sem_fechamento` | Texto de no sem fechamento. |
| `tipo_de_no_da_arvore_nao_suportado` | Tipo de no da árvore não suportado. |
| `vinculo_de_anexacao_fora_da_forma_observada` | Vínculo de anexação fora da forma observada. |
| `vinculos_de_anexacao_repetidos` | Vínculos de anexação repetidos. |

### arvore_processo/tela

Emissor: [src/arvore_processo/tela.rs](../../src/arvore_processo/tela.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `arvore_ausente_na_tela_do_processo` | Árvore ausente na tela do processo. |
| `arvores_repetidas_na_tela_do_processo` | Árvores repetidas na tela do processo. |
| `endereco_da_arvore_de_outro_processo` | Endereço da árvore de outro processo. |
| `endereco_da_arvore_nao_reconhecido` | Endereço da árvore não reconhecido. |
| `html_da_tela_do_processo_nao_interpretado` | HTML da tela do processo não interpretado. |
| `identificacao_da_unidade_ausente_na_tela_do_processo` | Identificação da unidade ausente na tela do processo. |
| `unidade_da_tela_diferente_da_solicitada` | Unidade da tela diferente da solicitada. |
