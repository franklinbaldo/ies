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

## Falhas tipadas

### caixa_detalhada

Emissor: [src/caixa_detalhada.rs](../../src/caixa_detalhada.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `cabecalho_detalhado_diferente_entre_paginas` | Cabecalho detalhado diferente entre paginas. |
| `cabecalho_processo_detalhado_ausente` | Cabecalho processo detalhado ausente. |
| `cabecalho_processo_detalhado_repetido` | Cabecalho processo detalhado repetido. |
| `classes_de_estado_diferentes_no_link_detalhado` | Classes de estado diferentes no link detalhado. |
| `colunas_da_linha_detalhada_diferentes` | Colunas da linha detalhada diferentes. |
| `contexto_detalhado_alterado` | Contexto detalhado alterado. |
| `faixa_detalhada_descontinua` | Faixa detalhada descontinua. |
| `faixa_detalhada_invalida` | Faixa detalhada invalida. |
| `faixa_detalhada_sem_intervalo_com_paginacao` | Faixa detalhada sem intervalo com paginacao. |
| `html_detalhado_nao_interpretado` | Html detalhado nao interpretado. |
| `identificador_detalhado_invalido` | Identificador detalhado invalido. |
| `identificador_detalhado_repetido` | Identificador detalhado repetido. |
| `legenda_detalhada_ausente` | Legenda detalhada ausente. |
| `legendas_detalhadas_repetidas` | Legendas detalhadas repetidas. |
| `limite_paginas_detalhadas` | Limite paginas detalhadas. |
| `link_do_processo_detalhado_ausente` | Link do processo detalhado ausente. |
| `links_do_processo_detalhado_repetidos` | Links do processo detalhado repetidos. |
| `modo_detalhado_nao_confirmado` | Modo detalhado nao confirmado. |
| `numero_detalhado_repetido` | Numero detalhado repetido. |
| `numero_do_processo_detalhado_ausente` | Numero do processo detalhado ausente. |
| `tabela_detalhada_ausente` | Tabela detalhada ausente. |
| `tabelas_detalhadas_repetidas` | Tabelas detalhadas repetidas. |
| `total_detalhado_alterado` | Total detalhado alterado. |
| `total_detalhado_nao_declarado` | Total detalhado nao declarado. |
| `unidade_da_detalhada_ausente` | Unidade da detalhada ausente. |
| `unidade_da_detalhada_diferente_da_solicitada` | Unidade da detalhada diferente da solicitada. |

### erro_web

Emissor: [src/erro_web.rs](../../src/erro_web.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `classes_de_estado_diferentes_no_link_do_processo` | Classes de estado diferentes no link do processo. |
| `contexto_divergente` | Contexto divergente. |
| `contexto_nao_verificavel` | Contexto nao verificavel. |
| `faixa_desconhecida` | Faixa desconhecida. |
| `hashes_de_criterios_gerados_repetidos_no_controle` | Hashes de criterios gerados repetidos no controle. |
| `hashes_de_criterios_recebidos_repetidos_no_controle` | Hashes de criterios recebidos repetidos no controle. |
| `html_invalido` | Html invalido. |
| `id_invalido` | Id invalido. |
| `id_repetido` | Id repetido. |
| `identificacao_da_unidade_ausente_no_controle` | Identificacao da unidade ausente no controle. |
| `identificacoes_da_unidade_repetidas_no_controle` | Identificacoes da unidade repetidas no controle. |
| `legenda_ausente` | Legenda ausente. |
| `legendas_da_tabela_repetidas` | Legendas da tabela repetidas. |
| `limite_paginas` | Limite paginas. |
| `link_do_processo_nao_identificado_na_linha` | Link do processo nao identificado na linha. |
| `numero_invalido` | Numero invalido. |
| `processo_ambiguo` | Processo ambiguo. |
| `sem_paginas` | Sem paginas. |
| `tabela_ausente` | Tabela ausente. |
| `tabela_invalida` | Tabela invalida. |
| `tabelas_da_categoria_repetidas` | Tabelas da categoria repetidas. |
| `tipos_de_visualizacao_repetidos_no_controle` | Tipos de visualizacao repetidos no controle. |
| `total_divergente` | Total divergente. |
| `total_excedido` | Total excedido. |
| `unidade_inesperada` | Unidade inesperada. |
| `visao_resumida_com_paginacao_nao_demonstrada` | Visao resumida com paginacao nao demonstrada. |

### formulario_web/erro

Emissor: [src/formulario_web/erro.rs](../../src/formulario_web/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `acao_do_formulario_ausente` | Acao do formulario ausente. |
| `acao_do_formulario_diferente_da_operacao` | Acao do formulario diferente da operacao. |
| `acao_do_formulario_fora_da_instalacao` | Acao do formulario fora da instalacao. |
| `acao_do_formulario_nao_eh_url_https` | Acao do formulario nao eh url https. |
| `acao_do_formulario_sem_infra_hash_unico` | Acao do formulario sem infra hash unico. |
| `accept_charset_publicado_no_formulario` | Accept charset publicado no formulario. |
| `botao_de_envio_ausente_no_formulario` | Botao de envio ausente no formulario. |
| `botoes_de_envio_repetidos_no_formulario` | Botoes de envio repetidos no formulario. |
| `campo_criado_pelo_script_ja_publicado` | Campo criado pelo script ja publicado. |
| `campo_fora_da_lista_da_operacao` | Campo fora da lista da operacao. |
| `campo_informado_mais_de_uma_vez` | Campo informado mais de uma vez. |
| `campo_nao_publicado_pelo_formulario` | Campo nao publicado pelo formulario. |
| `campo_oculto_ausente_no_formulario` | Campo oculto ausente no formulario. |
| `campo_oculto_repetido_no_formulario` | Campo oculto repetido no formulario. |
| `efeito_da_operacao_nao_avaliado` | Efeito da operacao nao avaliado. |
| `enctype_do_formulario_nao_urlencoded` | Enctype do formulario nao urlencoded. |
| `entidade_html_desconhecida_no_atributo` | Entidade html desconhecida no atributo. |
| `formulario_ausente_na_tela` | Formulario ausente na tela. |
| `formularios_com_mesmo_identificador_na_tela` | Formularios com mesmo identificador na tela. |
| `hdn_acao_publicado_sem_regra_da_operacao` | Hdn acao publicado sem regra da operacao. |
| `identificacao_da_unidade_ausente_na_tela` | Identificacao da unidade ausente na tela. |
| `identificacoes_da_unidade_repetidas_na_tela` | Identificacoes da unidade repetidas na tela. |
| `marcador_de_resultado_ausente_na_resposta` | Marcador de resultado ausente na resposta. |
| `metodo_do_formulario_diferente_de_post` | Metodo do formulario diferente de post. |
| `tela_de_login_recebida_apos_envio` | Tela de login recebida apos envio. |
| `validacao_recusada_pelo_sei` | Validacao recusada pelo sei. |
| `valor_nao_representavel_na_codificacao` | Valor nao representavel na codificacao. |

### html_atributos

Emissor: [src/html_atributos.rs](../../src/html_atributos.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `atributo_repetido_na_tag` | Atributo repetido na tag. |
| `fim_da_tag_de_abertura_ausente` | Fim da tag de abertura ausente. |
| `inicio_da_tag_de_abertura_ausente` | Inicio da tag de abertura ausente. |
| `valor_entre_aspas_sem_fechamento` | Valor entre aspas sem fechamento. |

### nivel_de_detalhe/tela

Emissor: [src/nivel_de_detalhe/tela.rs](../../src/nivel_de_detalhe/tela.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `campo_nao_previsto_no_formulario_do_nivel_de_detalhe` | Campo nao previsto no formulario do nivel de detalhe. |
| `html_do_nivel_de_detalhe_nao_interpretado` | Html do nivel de detalhe nao interpretado. |

### pagina_do_controle/regras

Emissor: [src/pagina_do_controle/regras.rs](../../src/pagina_do_controle/regras.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `entidade_html_desconhecida_no_controle` | Entidade html desconhecida no controle. |

### visao_do_controle

Emissor: [src/visao_do_controle.rs](../../src/visao_do_controle.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `visao_do_controle_nao_reconhecida` | Visao do controle nao reconhecida. |
