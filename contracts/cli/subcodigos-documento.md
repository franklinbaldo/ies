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

Contratos de operação: [Leitura](documento-obter.md), [versões](documento-obter-versoes.md), [anexos](documento-obter-externo.md) e [salvar](documento-salvar.md).

## Falhas tipadas

### anexo_documento

Emissor: [src/anexo_documento.rs](../../src/anexo_documento.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `sessao_encerrada_rode_ies_login` | Sessão encerrada rode IES login. |

### arquivo_texto_documento

Emissor: [src/arquivo_texto_documento.rs](../../src/arquivo_texto_documento.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `arquivo_texto_documento_ja_existe` | Arquivo texto documento já existe. |
| `destino_texto_documento_sem_nome` | Destino texto documento sem nome. |
| `escrita_texto_documento_falhou` | Escrita texto documento falhou. |

### arvore_processo/barra_do_documento

Emissor: [src/arvore_processo/barra_do_documento.rs](../../src/arvore_processo/barra_do_documento.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `link_das_versoes_de_outro_documento` | Link das versões de outro documento. |
| `link_das_versoes_nao_reconhecido` | Link das versões não reconhecido. |
| `link_das_versoes_repetido_na_barra` | Link das versões repetido na barra. |

### copia_listagem

Emissor: [src/copia_listagem.rs](../../src/copia_listagem.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `limite_copias_locais` | Limite cópias locais. |

### documento

Emissor: [src/documento.rs](../../src/documento.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `numero_documento_controle` | Número documento controle. |
| `numero_documento_nao_numerico` | Número documento não numérico. |
| `numero_documento_vazio` | Número documento vazio. |

### documento_html_privado_cli

Emissor: [src/documento_html_privado_cli.rs](../../src/documento_html_privado_cli.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `html_privado_exige_um_documento` | HTML privado exige um documento. |
| `numero_do_html_diferente_do_solicitado` | Número do HTML diferente do solicitado. |
| `opcoes_de_documento_incompativeis_com_html_privado` | Opções de documento incompatíveis com HTML privado. |

### documento_versoes

Emissor: [src/documento_versoes.rs](../../src/documento_versoes.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `historicos_documento_duplicados` | Históricos documento duplicados. |

### documento_web

Emissor: [src/documento_web.rs](../../src/documento_web.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `arvore_documento_aninhamento_excessivo` | Árvore documento aninhamento excessivo. |
| `cabecalhos_html_repetidos` | Cabeçalhos HTML repetidos. |
| `corpo_documento_ausente` | Corpo documento ausente. |
| `corpo_documento_contem_script_ou_estilo` | Corpo documento contém script ou estilo. |
| `corpo_documento_vazio` | Corpo documento vazio. |
| `corpos_de_documento_repetidos` | Corpos de documento repetidos. |
| `html_corpo_documento_nao_interpretado` | HTML corpo documento não interpretado. |
| `html_documento_nao_interpretado` | HTML documento não interpretado. |
| `texto_documento_acima_do_limite` | Texto documento acima do limite. |
| `titulo_documento_ausente` | Título documento ausente. |
| `titulo_documento_sem_identificacao_sei` | Título documento sem identificação SEI. |
| `titulo_documento_sem_numero_sei` | Título documento sem número SEI. |
| `titulo_documento_sem_tipo` | Título documento sem tipo. |
| `titulos_de_documento_repetidos` | Títulos de documento repetidos. |

### propriedades_documento

Emissor: [src/propriedades_documento.rs](../../src/propriedades_documento.rs).
Somente leitura do link assinado e do cadastro publicado; não há POST.

| Subcódigo | Fato observado |
| --- | --- |
| `barra_cadastro_documento_ilegivel` | Barra do nó de documento não reconhecida. |
| `cadastro_documento_ilegivel` | HTML ou atributo do cadastro malformado. |
| `campo_cadastro_documento_ausente` | Controle esperado não publicado. |
| `campo_cadastro_documento_repetido` | Controle identificado mais de uma vez. |
| `formulario_cadastro_documento_ausente` | Não existe formulário observado. |
| `formularios_cadastro_documento_repetidos` | Formulário publicado mais de uma vez. |
| `identidade_cadastro_documento_divergente` | IDs ocultos diferem dos da árvore. |
| `link_cadastro_documento_invalido` | Link assinado diverge da ação ou dos IDs. |
| `links_cadastro_documento_repetidos` | Barra publica vários links de cadastro. |
| `nivel_cadastro_documento_ilegivel` | Opções de nível ilegíveis. |
| `nivel_cadastro_documento_inesperado` | Marcação de nível não reconhecida. |
| `texto_cadastro_documento_ilegivel` | Texto não decodificável. |
| `tipo_cadastro_documento_vazio` | Tipo não publicado no rótulo do cadastro. |
| `valor_cadastro_documento_ausente` | Campo sem valor esperado. |
