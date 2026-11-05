---
type: Contract
title: Subcódigos de Controle de Processos
description: Inventário literal complementar de falhas por contexto emissor
---

# Subcódigos de Controle de Processos

Este complemento enumera as condições de listas, visão detalhada, paginação e nível de detalhe; atributos HTML são compartilhados com outros leitores.
As tabelas completam os contratos das operações; não mudam códigos de
saída, orçamento nem regras de recuperação. Delegações preservam o
subcódigo do leitor chamado. O mesmo literal pode aparecer em `subcodigo`
ou em uma falha aninhada; interpretar junto à operação e à etapa.
Ver [regras e contextos compartilhados](../../docs/erros-tipados.md).

Contratos de operação: [Listas](unidade-listar-processos.md), [visão detalhada](unidade-listar-processos-visao-detalhada.md), [paginação](unidade-listar-processos-paginas.md) e [colunas](unidade-colunas.md).

## Falhas tipadas

### caixa_detalhada

Emissor: [src/caixa_detalhada.rs](../../src/caixa_detalhada.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `cabecalho_detalhado_diferente_entre_paginas` | Cabeçalho detalhado diferente entre páginas. |
| `cabecalho_processo_detalhado_ausente` | Cabeçalho processo detalhado ausente. |
| `cabecalho_processo_detalhado_repetido` | Cabeçalho processo detalhado repetido. |
| `classes_de_estado_diferentes_no_link_detalhado` | Classes de estado diferentes no link detalhado. |
| `colunas_da_linha_detalhada_diferentes` | Colunas da linha detalhada diferentes. |
| `contexto_detalhado_alterado` | Contexto detalhado alterado. |
| `faixa_detalhada_descontinua` | Faixa detalhada descontínua. |
| `faixa_detalhada_invalida` | Faixa detalhada inválida. |
| `faixa_detalhada_sem_intervalo_com_paginacao` | Faixa detalhada sem intervalo com paginação. |
| `html_detalhado_nao_interpretado` | HTML detalhado não interpretado. |
| `identificador_detalhado_invalido` | Identificador detalhado inválido. |
| `identificador_detalhado_repetido` | Identificador detalhado repetido. |
| `legenda_detalhada_ausente` | Legenda detalhada ausente. |
| `legendas_detalhadas_repetidas` | Legendas detalhadas repetidas. |
| `limite_paginas_detalhadas` | Limite páginas detalhadas. |
| `link_do_processo_detalhado_ausente` | Link do processo detalhado ausente. |
| `links_do_processo_detalhado_repetidos` | Links do processo detalhado repetidos. |
| `modo_detalhado_nao_confirmado` | Modo detalhado não confirmado. |
| `numero_detalhado_repetido` | Número detalhado repetido. |
| `numero_do_processo_detalhado_ausente` | Número do processo detalhado ausente. |
| `tabela_detalhada_ausente` | Tabela detalhada ausente. |
| `tabelas_detalhadas_repetidas` | Tabelas detalhadas repetidas. |
| `total_detalhado_alterado` | Total detalhado alterado. |
| `total_detalhado_nao_declarado` | Total detalhado não declarado. |
| `unidade_da_detalhada_ausente` | Unidade da detalhada ausente. |
| `unidade_da_detalhada_diferente_da_solicitada` | Unidade da detalhada diferente da solicitada. |

### erro_web

Emissor: [src/erro_web.rs](../../src/erro_web.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `classes_de_estado_diferentes_no_link_do_processo` | Classes de estado diferentes no link do processo. |
| `contexto_divergente` | Contexto divergente. |
| `contexto_nao_verificavel` | Contexto não verificável. |
| `faixa_desconhecida` | Faixa desconhecida. |
| `hashes_de_criterios_gerados_repetidos_no_controle` | Hashes de critérios gerados repetidos no controle. |
| `hashes_de_criterios_recebidos_repetidos_no_controle` | Hashes de critérios recebidos repetidos no controle. |
| `html_da_caixa_invalido` | HTML inválido. |
| `id_invalido` | ID inválido. |
| `id_repetido` | ID repetido. |
| `identificacao_da_unidade_ausente_no_controle` | Identificação da unidade ausente no controle. |
| `identificacoes_da_unidade_repetidas_no_controle` | Identificações da unidade repetidas no controle. |
| `legenda_ausente` | Legenda ausente. |
| `legendas_da_tabela_repetidas` | Legendas da tabela repetidas. |
| `limite_paginas` | Limite páginas. |
| `link_do_processo_nao_identificado_na_linha` | Link do processo não identificado na linha. |
| `numero_da_caixa_invalido` | Número inválido. |
| `processo_ambiguo` | Processo ambíguo. |
| `sem_paginas` | Sem páginas. |
| `tabela_ausente` | Tabela ausente. |
| `tabela_invalida` | Tabela inválida. |
| `tabelas_da_categoria_repetidas` | Tabelas da categoria repetidas. |
| `tipos_de_visualizacao_repetidos_no_controle` | Tipos de visualização repetidos no controle. |
| `total_divergente` | Total divergente. |
| `total_excedido` | Total excedido. |
| `unidade_inesperada` | Unidade inesperada. |
| `visao_resumida_com_paginacao_nao_demonstrada` | Visão resumida com paginação não demonstrada. |

### formulario_web/erro

Emissor: [src/formulario_web/erro.rs](../../src/formulario_web/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `acao_do_formulario_ausente` | Ação do formulário ausente. |
| `acao_do_formulario_diferente_da_operacao` | Ação do formulário diferente da operação. |
| `acao_do_formulario_fora_da_instalacao` | Ação do formulário fora da instalação. |
| `acao_do_formulario_nao_eh_url_https` | Ação do formulário não é URL HTTPS. |
| `acao_do_formulario_sem_infra_hash_unico` | Ação do formulário sem infra hash único. |
| `accept_charset_publicado_no_formulario` | Accept charset publicado no formulário. |
| `botao_de_envio_ausente_no_formulario` | Botão de envio ausente no formulário. |
| `botoes_de_envio_repetidos_no_formulario` | Botões de envio repetidos no formulário. |
| `campo_criado_pelo_script_ja_publicado` | Campo criado pelo script já publicado. |
| `campo_fora_da_lista_da_operacao` | Campo fora da lista da operação. |
| `campo_informado_mais_de_uma_vez` | Campo informado mais de uma vez. |
| `campo_nao_publicado_pelo_formulario` | Campo não publicado pelo formulário. |
| `campo_oculto_ausente_no_formulario` | Campo oculto ausente no formulário. |
| `campo_oculto_repetido_no_formulario` | Campo oculto repetido no formulário. |
| `efeito_da_operacao_nao_avaliado` | Efeito da operação não avaliado. |
| `enctype_do_formulario_nao_urlencoded` | Enctype do formulário não urlencoded. |
| `entidade_html_desconhecida_no_atributo` | Entidade HTML desconhecida no atributo. |
| `formulario_ausente_na_tela` | Formulário ausente na tela. |
| `formularios_com_mesmo_identificador_na_tela` | Formulários com mesmo identificador na tela. |
| `hdn_acao_publicado_sem_regra_da_operacao` | Hdn ação publicado sem regra da operação. |
| `identificacao_da_unidade_ausente_na_tela` | Identificação da unidade ausente na tela. |
| `identificacoes_da_unidade_repetidas_na_tela` | Identificações da unidade repetidas na tela. |
| `marcador_de_resultado_ausente_na_resposta` | Marcador de resultado ausente na resposta. |
| `metodo_do_formulario_diferente_de_post` | Método do formulário diferente de POST. |
| `tela_de_login_recebida_apos_envio` | Tela de login recebida após envio. |
| `validacao_recusada_pelo_sei` | Validação recusada pelo SEI. |
| `valor_nao_representavel_na_codificacao` | Valor não representável na codificação. |

### html_atributos

Emissor: [src/html_atributos.rs](../../src/html_atributos.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `atributo_repetido_na_tag` | Atributo repetido na tag. |
| `fim_da_tag_de_abertura_ausente` | Fim da tag de abertura ausente. |
| `inicio_da_tag_de_abertura_ausente` | Início da tag de abertura ausente. |
| `valor_entre_aspas_sem_fechamento` | Valor entre aspas sem fechamento. |

### nivel_de_detalhe/tela

Emissor: [src/nivel_de_detalhe/tela.rs](../../src/nivel_de_detalhe/tela.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `campo_nao_previsto_no_formulario_do_nivel_de_detalhe` | Campo não previsto no formulário do nível de detalhe. |
| `html_do_nivel_de_detalhe_nao_interpretado` | HTML do nível de detalhe não interpretado. |

### pagina_do_controle/regras

Emissor: [src/pagina_do_controle/regras.rs](../../src/pagina_do_controle/regras.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `entidade_html_desconhecida_no_controle` | Entidade HTML desconhecida no controle. |
| `pagina_da_caixa_fora_do_intervalo` | Página da caixa fora do intervalo observado. |

### visao_do_controle

Emissor: [src/visao_do_controle.rs](../../src/visao_do_controle.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `visao_do_controle_nao_reconhecida` | Visão do controle não reconhecida. |
