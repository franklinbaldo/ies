---
type: Contract
title: Subcódigos de pesquisa Web
description: Inventário literal complementar de falhas por contexto emissor
---

# Subcódigos de pesquisa Web

Este complemento enumera as condições de `processo buscar` e `documento buscar`, incluindo formulário, resultado e paginação.
As tabelas completam os contratos das operações; não mudam códigos de
saída, orçamento nem regras de recuperação. Delegações preservam o
subcódigo do leitor chamado. O mesmo literal pode aparecer em `subcodigo`
ou em uma falha aninhada; interpretar junto à operação e à etapa.
Ver [regras e contextos compartilhados](../../docs/erros-tipados.md).

Contratos de operação: [Processos](processo-buscar.md) e [documentos](documento-buscar.md).

## Falhas tipadas

### documento_buscar_cli

Emissor: [src/documento_buscar_cli.rs](../../src/documento_buscar_cli.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `termo_de_documento_contem_controle` | Termo de documento contém controle. |
| `termo_de_documento_vazio` | Termo de documento vazio. |

### pesquisa_web

Emissor: [src/pesquisa_web.rs](../../src/pesquisa_web.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `html_de_pesquisa_nao_interpretado` | HTML de pesquisa não interpretado. |
| `identificador_do_processo_repetido_na_pesquisa` | Identificador do processo repetido na pesquisa. |
| `linha_de_pesquisa_com_mais_de_um_link_da_arvore` | Linha de pesquisa com mais de um link da árvore. |
| `linha_de_pesquisa_com_mais_de_um_link_do_processo` | Linha de pesquisa com mais de um link do processo. |
| `linha_de_pesquisa_sem_link_do_processo` | Linha de pesquisa sem link do processo. |
| `link_da_arvore_com_href_diferente_do_link_do_processo` | Link da árvore com href diferente do link do processo. |
| `link_de_pesquisa_com_acao_sem_igual` | Link de pesquisa com ação sem igual. |
| `link_de_pesquisa_com_chave_nao_canonica` | Link de pesquisa com chave não canônica. |
| `link_de_pesquisa_com_identificador_do_processo_nao_decimal` | Link de pesquisa com identificador do processo não decimal. |
| `link_de_pesquisa_com_identificador_sem_igual` | Link de pesquisa com identificador sem igual. |
| `link_de_pesquisa_com_identificadores_repetidos` | Link de pesquisa com identificadores repetidos. |
| `link_de_pesquisa_nao_abre_processo` | Link de pesquisa não abre processo. |
| `link_de_pesquisa_sem_identificador_do_processo` | Link de pesquisa sem identificador do processo. |
| `link_de_processo_fora_da_linha_de_pesquisa` | Link de processo fora da linha de pesquisa. |
| `link_de_processo_sem_classe_reconhecida` | Link de processo sem classe reconhecida. |
| `link_do_documento_da_linha_diferente_do_processo` | Link do documento da linha diferente do processo. |
| `numero_de_pesquisa_repetido` | Número de pesquisa repetido. |
| `tabela_de_pesquisa_ausente` | Tabela de pesquisa ausente. |
| `tabelas_de_pesquisa_repetidas` | Tabelas de pesquisa repetidas. |

### pesquisa_web/documentos

Emissor: [src/pesquisa_web/documentos.rs](../../src/pesquisa_web/documentos.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `celula_do_titulo_do_documento_ausente_ou_repetida` | Célula do título do documento ausente ou repetida. |
| `data_de_inclusao_fora_da_forma_observada` | Data de inclusão fora da forma observada. |
| `linha_da_pesquisa_fora_de_um_registro` | Linha da pesquisa fora de um registro. |
| `linha_de_documento_sem_id_documento_no_link_do_processo` | Linha de documento sem ID documento no link do processo. |
| `link_da_linha_de_documento_nao_reconhecido` | Link da linha de documento não reconhecido. |
| `link_do_documento_ausente_na_linha` | Link do documento ausente na linha. |
| `link_do_documento_diferente_do_processo` | Link do documento diferente do processo. |
| `links_do_documento_repetidos_na_linha` | Links do documento repetidos na linha. |
| `metadado_do_documento_repetido` | Metadado do documento repetido. |
| `numero_sei_ausente_na_linha` | Número SEI ausente na linha. |
| `numero_sei_com_link_diferente_do_documento` | Número SEI com link diferente do documento. |
| `numero_sei_repetido_na_pesquisa` | Número SEI repetido na pesquisa. |
| `rotulo_do_documento_ausente` | Rótulo do documento ausente. |

### pesquisa_web/erro_busca

Emissor: [src/pesquisa_web/erro_busca.rs](../../src/pesquisa_web/erro_busca.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `charset_da_tela_da_pesquisa_nao_suportado` | Charset da tela da pesquisa não suportado. |

### pesquisa_web/link_da_pesquisa

Emissor: [src/pesquisa_web/link_da_pesquisa.rs](../../src/pesquisa_web/link_da_pesquisa.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `link_da_pesquisa_ausente_no_controle` | Link da pesquisa ausente no controle. |
| `link_da_pesquisa_com_endereco_absoluto` | Link da pesquisa com endereço absoluto. |
| `link_da_pesquisa_nao_reconhecido` | Link da pesquisa não reconhecido. |
| `links_da_pesquisa_repetidos_no_controle` | Links da pesquisa repetidos no controle. |

### pesquisa_web/paginacao

Emissor: [src/pesquisa_web/paginacao.rs](../../src/pesquisa_web/paginacao.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `faixa_da_pesquisa_diferente_da_pagina_solicitada` | Faixa da pesquisa diferente da página solicitada. |
| `html_da_tela_de_resultado_da_pesquisa_nao_interpretado` | HTML da tela de resultado da pesquisa não interpretado. |
| `link_de_paginacao_da_pesquisa_fora_da_regra` | Link de paginação da pesquisa fora da regra. |
| `pagina_da_pesquisa_fora_do_intervalo` | Página da pesquisa fora do intervalo observado. |
| `pagina_da_pesquisa_sem_resultado_apos_paginar` | Página da pesquisa sem resultado após paginar. |
| `paginador_da_pesquisa_nao_publicado` | Paginador da pesquisa não publicado. |
| `protocolo_da_tela_de_resultado_diferente_do_pesquisado` | Protocolo da tela de resultado diferente do pesquisado. |
| `redirecionamento_ao_paginar_a_pesquisa` | Redirecionamento ao paginar a pesquisa. |
| `regra_de_navegacao_da_pesquisa_nao_observada` | Regra de navegação da pesquisa não observada. |
| `total_da_pesquisa_alterado_apos_paginar` | Total da pesquisa alterado após paginar. |

### pesquisa_web/redirecionamento

Emissor: [src/pesquisa_web/redirecionamento.rs](../../src/pesquisa_web/redirecionamento.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `acao_de_origem_ausente_no_redirecionamento` | Ação de origem ausente no redirecionamento. |
| `acao_de_origem_do_redirecionamento_diferente_da_pesquisa` | Ação de origem do redirecionamento diferente da pesquisa. |
| `acao_de_origem_inesperada_no_redirecionamento` | Ação de origem inesperada no redirecionamento. |
| `acao_de_origem_repetida_no_redirecionamento` | Ação de origem repetida no redirecionamento. |
| `id_documento_ausente_no_redirecionamento` | ID documento ausente no redirecionamento. |
| `id_documento_nao_decimal_no_redirecionamento` | ID documento não decimal no redirecionamento. |
| `id_documento_repetido_no_redirecionamento` | ID documento repetido no redirecionamento. |
| `id_procedimento_ausente_no_redirecionamento` | ID procedimento ausente no redirecionamento. |
| `id_procedimento_do_redirecionamento_diferente_do_pedido` | ID procedimento do redirecionamento diferente do pedido. |
| `id_procedimento_nao_decimal_no_redirecionamento` | ID procedimento não decimal no redirecionamento. |
| `id_procedimento_repetido_no_redirecionamento` | ID procedimento repetido no redirecionamento. |
| `infra_hash_ausente_no_redirecionamento` | Infra hash ausente no redirecionamento. |
| `infra_hash_fora_do_formato_no_redirecionamento` | Infra hash fora do formato no redirecionamento. |
| `infra_hash_repetido_no_redirecionamento` | Infra hash repetido no redirecionamento. |
| `infra_sistema_ausente_na_url_do_envio` | Infra sistema ausente na URL do envio. |
| `infra_sistema_ausente_no_redirecionamento` | Infra sistema ausente no redirecionamento. |
| `infra_sistema_do_redirecionamento_diferente_do_envio` | Infra sistema do redirecionamento diferente do envio. |
| `infra_sistema_nao_decimal_na_url_do_envio` | Infra sistema não decimal na URL do envio. |
| `infra_sistema_nao_decimal_no_redirecionamento` | Infra sistema não decimal no redirecionamento. |
| `infra_sistema_repetido_na_url_do_envio` | Infra sistema repetido na URL do envio. |
| `infra_sistema_repetido_no_redirecionamento` | Infra sistema repetido no redirecionamento. |
| `localizacao_do_redirecionamento_acima_do_limite` | Localização do redirecionamento acima do limite. |
| `localizacao_do_redirecionamento_ausente` | Localização do redirecionamento ausente. |
| `localizacao_do_redirecionamento_fora_da_instalacao` | Localização do redirecionamento fora da instalação. |
| `localizacao_do_redirecionamento_nao_eh_url_https` | Localização do redirecionamento não é URL HTTPS. |
| `parametro_nao_canonico_na_localizacao_do_redirecionamento` | Parâmetro não canônico na localização do redirecionamento. |
| `redirecionamento_para_acao_diferente_de_procedimento_trabalhar` | Redirecionamento para ação diferente de procedimento trabalhar. |
| `unidade_atual_ausente_na_url_do_envio` | Unidade atual ausente na URL do envio. |
| `unidade_atual_ausente_no_redirecionamento` | Unidade atual ausente no redirecionamento. |
| `unidade_atual_do_redirecionamento_diferente_da_do_envio` | Unidade atual do redirecionamento diferente da do envio. |
| `unidade_atual_repetida_na_url_do_envio` | Unidade atual repetida na URL do envio. |
| `unidade_atual_repetida_no_redirecionamento` | Unidade atual repetida no redirecionamento. |

### pesquisa_web/regras_formulario

Emissor: [src/pesquisa_web/regras_formulario.rs](../../src/pesquisa_web/regras_formulario.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `assinante_oculto_preenchido_na_pesquisa_em_documentos` | Assinante oculto preenchido na pesquisa em documentos. |
| `assinante_oculto_preenchido_na_pesquisa_em_processos` | Assinante oculto preenchido na pesquisa em processos. |
| `caixa_de_tipo_de_documento_ausente_na_tela` | Caixa de tipo de documento ausente na tela. |
| `caixa_de_tipo_de_documento_sem_valor_na_tela` | Caixa de tipo de documento sem valor na tela. |
| `caixas_de_tipo_de_documento_repetidas_na_tela` | Caixas de tipo de documento repetidas na tela. |
| `campos_do_termo_repetidos_na_tela` | Campos do termo repetidos na tela. |
| `entidade_html_desconhecida_na_regra` | Entidade HTML desconhecida na regra. |
| `inicio_da_paginacao_diferente_de_zero_na_tela` | Início da paginação diferente de zero na tela. |
| `maxlength_do_termo_ilegivel_na_tela` | Maxlength do termo ilegível na tela. |
| `nenhum_tipo_de_documento_marcado_na_tela` | Nenhum tipo de documento marcado na tela. |
| `opcao_de_documentos_ausente_na_tela` | Opção de documentos ausente na tela. |
| `opcao_de_documentos_com_nome_diferente_de_rdo_pesquisar_em` | Opção de documentos com nome diferente de rdo pesquisar em. |
| `opcao_de_documentos_sem_valor_na_tela` | Opção de documentos sem valor na tela. |
| `opcao_de_orgao_selecionada_sem_valor_na_tela` | Opção de órgão selecionada sem valor na tela. |
| `opcao_de_processos_ausente_na_tela` | Opção de processos ausente na tela. |
| `opcao_de_processos_com_nome_diferente_de_rdo_pesquisar_em` | Opção de processos com nome diferente de rdo pesquisar em. |
| `opcao_de_processos_sem_valor_na_tela` | Opção de processos sem valor na tela. |
| `opcoes_de_documentos_repetidas_na_tela` | Opções de documentos repetidas na tela. |
| `opcoes_de_processos_repetidas_na_tela` | Opções de processos repetidas na tela. |
| `orgao_de_pesquisa_sem_opcao_selecionada_na_tela` | Órgão de pesquisa sem opção selecionada na tela. |
| `seletor_de_orgao_de_pesquisa_ausente_na_tela` | Seletor de órgão de pesquisa ausente na tela. |
| `seletor_de_orgao_de_pesquisa_sem_multipla_escolha` | Seletor de órgão de pesquisa sem múltipla escolha. |
| `seletores_de_orgao_de_pesquisa_repetidos_na_tela` | Seletores de órgão de pesquisa repetidos na tela. |
| `termo_acima_do_maxlength_da_tela` | Termo acima do maxlength da tela. |

### pesquisa_web/resposta

Emissor: [src/pesquisa_web/resposta.rs](../../src/pesquisa_web/resposta.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `aviso_de_sem_resultado_e_barra_de_contagem_na_mesma_resposta` | Aviso de sem resultado e barra de contagem na mesma resposta. |
| `barra_de_contagem_da_pesquisa_ausente` | Barra de contagem da pesquisa ausente. |
| `barras_de_contagem_da_pesquisa_repetidas` | Barras de contagem da pesquisa repetidas. |
| `faixa_da_barra_de_contagem_impossivel` | Faixa da barra de contagem impossível. |
| `html_da_resposta_da_pesquisa_nao_interpretado` | HTML da resposta da pesquisa não interpretado. |
| `linhas_da_tabela_diferentes_da_faixa_da_barra` | Linhas da tabela diferentes da faixa da barra. |
| `numero_da_barra_de_contagem_fora_do_formato` | Número da barra de contagem fora do formato. |
| `resposta_da_pesquisa_nao_reconhecida` | Resposta da pesquisa não reconhecida. |
| `tabela_de_resultado_e_aviso_de_sem_resultado_na_mesma_resposta` | Tabela de resultado e aviso de sem resultado na mesma resposta. |
| `texto_da_barra_de_contagem_fora_do_formato` | Texto da barra de contagem fora do formato. |
