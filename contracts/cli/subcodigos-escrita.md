---
type: Contract
title: Subcódigos de escritas verificadas
description: Inventário literal complementar de falhas por contexto emissor
---

# Subcódigos de escritas verificadas

Este complemento enumera as condições de prévias, submissões e verificação de enviar, concluir, reabrir, assinar, trocar unidade e definir colunas.
As tabelas completam os contratos das operações; não mudam códigos de
saída, orçamento nem regras de recuperação. Delegações preservam o
subcódigo do leitor chamado. O mesmo literal pode aparecer em `subcodigo`
ou em uma falha aninhada; interpretar junto à operação e à etapa.
Ver [regras e contextos compartilhados](../../docs/erros-tipados.md).

## Falhas tipadas

### arvore_processo/assinatura

Emissor: [src/arvore_processo/assinatura.rs](../../src/arvore_processo/assinatura.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `acoes_de_assinar_repetidas_na_barra` | Acoes de assinar repetidas na barra. |
| `assinaturas_do_documento_fora_da_forma_observada` | Assinaturas do documento fora da forma observada. |
| `assinaturas_do_documento_repetidas_na_arvore` | Assinaturas do documento repetidas na arvore. |
| `barra_do_documento_fora_da_forma_observada` | Barra do documento fora da forma observada. |
| `barras_do_documento_repetidas_na_arvore` | Barras do documento repetidas na arvore. |
| `endereco_de_assinar_de_outro_documento` | Endereco de assinar de outro documento. |
| `endereco_de_assinar_nao_reconhecido` | Endereco de assinar nao reconhecido. |
| `link_de_assinar_ausente_na_tela_do_documento` | Link de assinar ausente na tela do documento. |
| `link_de_assinar_fora_da_forma_observada` | Link de assinar fora da forma observada. |
| `links_de_assinar_repetidos_na_tela_do_documento` | Links de assinar repetidos na tela do documento. |

### arvore_processo/concluir

Emissor: [src/arvore_processo/concluir.rs](../../src/arvore_processo/concluir.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `barra_do_processo_fora_da_forma_observada` | Barra do processo fora da forma observada. |
| `barras_do_processo_repetidas_na_arvore` | Barras do processo repetidas na arvore. |
| `endereco_da_conclusao_de_outro_processo` | Endereco da conclusao de outro processo. |
| `endereco_da_conclusao_nao_reconhecido` | Endereco da conclusao nao reconhecido. |
| `endereco_da_tela_do_no_nao_reconhecido` | Endereco da tela do no nao reconhecido. |
| `formas_de_concluir_repetidas_na_barra` | Formas de concluir repetidas na barra. |
| `funcao_de_concluir_ausente_na_tela_do_no` | Funcao de concluir ausente na tela do no. |
| `funcao_de_concluir_fora_da_forma_observada` | Funcao de concluir fora da forma observada. |
| `funcoes_de_concluir_repetidas_na_tela_do_no` | Funcoes de concluir repetidas na tela do no. |

### arvore_processo/enviar

Emissor: [src/arvore_processo/enviar.rs](../../src/arvore_processo/enviar.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `endereco_do_envio_de_outro_processo` | Endereco do envio de outro processo. |
| `endereco_do_envio_nao_reconhecido` | Endereco do envio nao reconhecido. |
| `links_de_enviar_repetidos_na_barra` | Links de enviar repetidos na barra. |

### arvore_processo/reabrir

Emissor: [src/arvore_processo/reabrir.rs](../../src/arvore_processo/reabrir.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `funcoes_de_reabrir_repetidas_na_barra` | Funcoes de reabrir repetidas na barra. |

### documento_assinar/erro

Emissor: [src/documento_assinar/erro.rs](../../src/documento_assinar/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `assinatura_nao_aplicada` | Assinatura nao aplicada. |
| `assinatura_nao_publicada_na_barra` | Assinatura nao publicada na barra. |
| `cargo_nao_oferecido_pela_tela` | Cargo nao oferecido pela tela. |
| `charset_da_tela_de_assinatura_nao_suportado` | Charset da tela de assinatura nao suportado. |
| `documento_assinado_por_outro_usuario` | Documento assinado por outro usuario. |
| `efeito_incerto_na_assinatura` | Efeito incerto na assinatura. |
| `endereco_da_tela_do_documento_nao_reconhecido` | Endereco da tela do documento nao reconhecido. |
| `redirecionamento_apos_assinar` | Redirecionamento apos assinar. |
| `resposta_da_assinatura_sem_finalizacao` | Resposta da assinatura sem finalizacao. |
| `tela_da_assinatura_redirecionada` | Tela da assinatura redirecionada. |

### processo_concluir/erro

Emissor: [src/processo_concluir/erro.rs](../../src/processo_concluir/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `campo_nao_previsto_na_tela_de_conclusao` | Campo nao previsto na tela de conclusao. |
| `charset_da_tela_de_conclusao_nao_suportado` | Charset da tela de conclusao nao suportado. |
| `conclusao_nao_aplicada` | Conclusao nao aplicada. |
| `conclusao_nao_publicada_na_barra` | Conclusao nao publicada na barra. |
| `efeito_incerto_na_conclusao` | Efeito incerto na conclusao. |
| `redirecionamento_na_conclusao_pela_funcao` | Redirecionamento na conclusao pela funcao. |
| `redirecionamento_na_reabertura` | Redirecionamento na reabertura. |
| `redirecionamento_nao_reconhecido_apos_concluir` | Redirecionamento nao reconhecido apos concluir. |
| `redirecionamento_nao_reconhecido_apos_enviar` | Redirecionamento nao reconhecido apos enviar. |
| `resposta_do_concluir_sem_redirecionamento` | Resposta do concluir sem redirecionamento. |
| `resposta_do_envio_sem_redirecionamento` | Resposta do envio sem redirecionamento. |
| `situacao_do_processo_nao_lida_antes_de_concluir` | Situacao do processo nao lida antes de concluir. |
| `tela_da_conclusao_redirecionada` | Tela da conclusao redirecionada. |
| `tela_de_conclusao_nao_interpretada` | Tela de conclusao nao interpretada. |
| `unidade_da_sessao_ausente_da_situacao` | Unidade da sessao ausente da situacao. |
| `unidade_da_sessao_nao_publicada_na_abertura` | Unidade da sessao nao publicada na abertura. |

### processo_enviar/erro

Emissor: [src/processo_enviar/erro.rs](../../src/processo_enviar/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `campo_nao_previsto_na_tela_de_envio` | Campo nao previsto na tela de envio. |
| `charset_da_tela_de_envio_nao_suportado` | Charset da tela de envio nao suportado. |
| `destino_nao_oferecido_pelo_sei` | Destino nao oferecido pelo sei. |
| `efeito_incerto_no_envio` | Efeito incerto no envio. |
| `envio_nao_aplicado` | Envio nao aplicado. |
| `envio_nao_publicado_na_barra` | Envio nao publicado na barra. |
| `processo_ja_aberto_no_destino` | Processo ja aberto no destino. |
| `processo_nao_aberto_na_unidade_da_sessao` | Processo nao aberto na unidade da sessao. |
| `resposta_do_autocompletar_fora_da_forma_observada` | Resposta do autocompletar fora da forma observada. |
| `resposta_do_autocompletar_nao_xml` | Resposta do autocompletar nao xml. |
| `situacao_da_origem_diferente_da_pedida_apos_envio` | Situacao da origem diferente da pedida apos envio. |
| `situacao_do_processo_nao_lida_antes_de_enviar` | Situacao do processo nao lida antes de enviar. |
| `tela_de_envio_de_outro_processo` | Tela de envio de outro processo. |
| `tela_de_envio_nao_interpretada` | Tela de envio nao interpretada. |
| `tela_de_envio_redirecionada` | Tela de envio redirecionada. |
| `varios_destinos_com_a_sigla` | Varios destinos com a sigla. |

### processo_reabrir/erro

Emissor: [src/processo_reabrir/erro.rs](../../src/processo_reabrir/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `efeito_incerto_na_reabertura` | Efeito incerto na reabertura. |
| `reabertura_nao_aplicada` | Reabertura nao aplicada. |
| `reabertura_nao_publicada_na_barra` | Reabertura nao publicada na barra. |
| `situacao_do_processo_nao_lida_antes_de_reabrir` | Situacao do processo nao lida antes de reabrir. |
| `tela_do_no_redirecionada` | Tela do no redirecionada. |
| `unidade_da_sessao_ja_na_situacao` | Unidade da sessao ja na situacao. |

### unidade_trocar/erro

Emissor: [src/unidade_trocar/erro.rs](../../src/unidade_trocar/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `efeito_incerto_na_troca_de_unidade` | Efeito incerto na troca de unidade. |
| `linha_de_unidade_fora_da_forma_observada` | Linha de unidade fora da forma observada. |
| `link_de_troca_de_unidade_ausente` | Link de troca de unidade ausente. |
| `link_de_troca_de_unidade_fora_da_forma_observada` | Link de troca de unidade fora da forma observada. |
| `links_de_troca_de_unidade_divergentes` | Links de troca de unidade divergentes. |
| `lista_de_unidades_incompleta` | Lista de unidades incompleta. |
| `localizacao_de_outra_unidade` | Localizacao de outra unidade. |
| `localizacao_fora_da_forma_observada` | Localizacao fora da forma observada. |
| `resposta_sem_redirecionamento` | Resposta sem redirecionamento. |
| `selecao_de_unidade_por_script_nao_publicada` | Selecao de unidade por script nao publicada. |
| `sigla_de_unidade_ambigua` | Sigla de unidade ambigua. |
| `status_inesperado` | Status inesperado. |
| `tela_de_troca_de_unidade_nao_reconhecida` | Tela de troca de unidade nao reconhecida. |
| `unidade_do_controle_diferente_da_pedida` | Unidade do controle diferente da pedida. |
| `unidade_marcada_diferente_da_ativa` | Unidade marcada diferente da ativa. |
| `unidade_nao_oferecida` | Unidade nao oferecida. |

### unidade_trocar_cli

Emissor: [src/unidade_trocar_cli.rs](../../src/unidade_trocar_cli.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `sigla_de_unidade_vazia` | Sigla de unidade vazia. |

### visao_do_controle/definir_erro

Emissor: [src/visao_do_controle/definir_erro.rs](../../src/visao_do_controle/definir_erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `coluna_indisponivel_nesta_instalacao` | Coluna indisponivel nesta instalacao. |
| `configuracao_de_colunas_nao_aplicada` | Configuracao de colunas nao aplicada. |
| `efeito_incerto_na_configuracao_de_colunas` | Efeito incerto na configuracao de colunas. |
| `marcas_do_nivel_de_detalhe_diferentes_do_alvo_apos_salvar` | Marcas do nivel de detalhe diferentes do alvo apos salvar. |
| `nivel_de_detalhe_devolvido_sem_sinal_de_gravacao` | Nivel de detalhe devolvido sem sinal de gravacao. |
| `redirecionamento_ao_salvar_nivel_de_detalhe` | Redirecionamento ao salvar nivel de detalhe. |
| `resposta_do_salvar_nao_reconhecida` | Resposta do salvar nao reconhecida. |
