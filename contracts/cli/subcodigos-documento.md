---
type: Contract
title: Subcódigos de documentos
description: Inventário literal complementar de falhas por contexto emissor
---

# Subcódigos de documentos

Este complemento enumera as condições de leitura Web e HTML privado, versões, anexos e cópias locais de documentos.
As tabelas completam os contratos das operações; não mudam códigos de
saída, orçamento nem regras de recuperação. Delegações preservam o
subcódigo do leitor chamado. O mesmo literal pode aparecer em `subcodigo`
ou em uma falha aninhada; interpretar junto à operação e à etapa.
Ver [regras e contextos compartilhados](../../docs/erros-tipados.md).

## Falhas tipadas

### anexo_documento

Emissor: [src/anexo_documento.rs](../../src/anexo_documento.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `sessao_encerrada_rode_ies_login` | Sessao encerrada rode ies login. |

### arquivo_texto_documento

Emissor: [src/arquivo_texto_documento.rs](../../src/arquivo_texto_documento.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `arquivo_texto_documento_ja_existe` | Arquivo texto documento ja existe. |
| `destino_texto_documento_sem_nome` | Destino texto documento sem nome. |
| `escrita_texto_documento_falhou` | Escrita texto documento falhou. |

### arvore_processo/barra_do_documento

Emissor: [src/arvore_processo/barra_do_documento.rs](../../src/arvore_processo/barra_do_documento.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `link_das_versoes_de_outro_documento` | Link das versoes de outro documento. |
| `link_das_versoes_nao_reconhecido` | Link das versoes nao reconhecido. |
| `link_das_versoes_repetido_na_barra` | Link das versoes repetido na barra. |

### copia_listagem

Emissor: [src/copia_listagem.rs](../../src/copia_listagem.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `limite_copias_locais` | Limite copias locais. |

### documento

Emissor: [src/documento.rs](../../src/documento.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `numero_documento_controle` | Numero documento controle. |
| `numero_documento_nao_numerico` | Numero documento nao numerico. |
| `numero_documento_vazio` | Numero documento vazio. |

### documento_html_privado_cli

Emissor: [src/documento_html_privado_cli.rs](../../src/documento_html_privado_cli.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `html_privado_exige_um_documento` | Html privado exige um documento. |
| `numero_do_html_diferente_do_solicitado` | Numero do html diferente do solicitado. |
| `opcoes_de_documento_incompativeis_com_html_privado` | Opcoes de documento incompativeis com html privado. |

### documento_versoes

Emissor: [src/documento_versoes.rs](../../src/documento_versoes.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `historicos_documento_duplicados` | Historicos documento duplicados. |

### documento_web

Emissor: [src/documento_web.rs](../../src/documento_web.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `arvore_documento_aninhamento_excessivo` | Arvore documento aninhamento excessivo. |
| `cabecalhos_html_repetidos` | Cabecalhos html repetidos. |
| `corpo_documento_ausente` | Corpo documento ausente. |
| `corpo_documento_contem_script_ou_estilo` | Corpo documento contem script ou estilo. |
| `corpo_documento_vazio` | Corpo documento vazio. |
| `corpos_de_documento_repetidos` | Corpos de documento repetidos. |
| `html_corpo_documento_nao_interpretado` | Html corpo documento nao interpretado. |
| `html_documento_nao_interpretado` | Html documento nao interpretado. |
| `texto_documento_acima_do_limite` | Texto documento acima do limite. |
| `titulo_documento_ausente` | Titulo documento ausente. |
| `titulo_documento_sem_identificacao_sei` | Titulo documento sem identificacao sei. |
| `titulo_documento_sem_numero_sei` | Titulo documento sem numero sei. |
| `titulo_documento_sem_tipo` | Titulo documento sem tipo. |
| `titulos_de_documento_repetidos` | Titulos de documento repetidos. |
