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

Contratos de operação: [Enviar](processo-enviar.md), [concluir](processo-concluir.md), [reabrir](processo-reabrir.md), [assinar](documento-assinar.md), [trocar unidade](unidade-trocar.md) e [definir colunas](unidade-definir-colunas.md).

## Falhas tipadas

### arvore_processo/assinatura

Emissor: [src/arvore_processo/assinatura.rs](../../src/arvore_processo/assinatura.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `acoes_de_assinar_repetidas_na_barra` | Ações de assinar repetidas na barra. |
| `assinaturas_do_documento_fora_da_forma_observada` | Assinaturas do documento fora da forma observada. |
| `assinaturas_do_documento_repetidas_na_arvore` | Assinaturas do documento repetidas na árvore. |
| `barra_do_documento_fora_da_forma_observada` | Barra do documento fora da forma observada. |
| `barras_do_documento_repetidas_na_arvore` | Barras do documento repetidas na árvore. |
| `endereco_de_assinar_de_outro_documento` | Endereço de assinar de outro documento. |
| `endereco_de_assinar_nao_reconhecido` | Endereço de assinar não reconhecido. |
| `link_de_assinar_ausente_na_tela_do_documento` | Link de assinar ausente na tela do documento. |
| `link_de_assinar_fora_da_forma_observada` | Link de assinar fora da forma observada. |
| `links_de_assinar_repetidos_na_tela_do_documento` | Links de assinar repetidos na tela do documento. |

### arvore_processo/concluir

Emissor: [src/arvore_processo/concluir.rs](../../src/arvore_processo/concluir.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `barra_do_processo_fora_da_forma_observada` | Barra do processo fora da forma observada. |
| `barras_do_processo_repetidas_na_arvore` | Barras do processo repetidas na árvore. |
| `endereco_da_conclusao_de_outro_processo` | Endereço da conclusão de outro processo. |
| `endereco_da_conclusao_nao_reconhecido` | Endereço da conclusão não reconhecido. |
| `endereco_da_tela_do_no_nao_reconhecido` | Endereço da tela do no não reconhecido. |
| `formas_de_concluir_repetidas_na_barra` | Formas de concluir repetidas na barra. |
| `funcao_de_concluir_ausente_na_tela_do_no` | Função de concluir ausente na tela do no. |
| `funcao_de_concluir_fora_da_forma_observada` | Função de concluir fora da forma observada. |
| `funcoes_de_concluir_repetidas_na_tela_do_no` | Funções de concluir repetidas na tela do no. |

### arvore_processo/enviar

Emissor: [src/arvore_processo/enviar.rs](../../src/arvore_processo/enviar.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `endereco_do_envio_de_outro_processo` | Endereço do envio de outro processo. |
| `endereco_do_envio_nao_reconhecido` | Endereço do envio não reconhecido. |
| `links_de_enviar_repetidos_na_barra` | Links de enviar repetidos na barra. |

### arvore_processo/reabrir

Emissor: [src/arvore_processo/reabrir.rs](../../src/arvore_processo/reabrir.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `funcoes_de_reabrir_repetidas_na_barra` | Funções de reabrir repetidas na barra. |

### documento_assinar/erro

Emissor: [src/documento_assinar/erro.rs](../../src/documento_assinar/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `assinatura_nao_aplicada` | Assinatura não aplicada. |
| `assinatura_nao_publicada_na_barra` | Assinatura não publicada na barra. |
| `cargo_nao_oferecido_pela_tela` | Cargo não oferecido pela tela. |
| `charset_da_tela_de_assinatura_nao_suportado` | Charset da tela de assinatura não suportado. |
| `documento_assinado_por_outro_usuario` | Documento assinado por outro usuário. |
| `efeito_incerto_na_assinatura` | Efeito incerto na assinatura. |
| `endereco_da_tela_do_documento_nao_reconhecido` | Endereço da tela do documento não reconhecido. |
| `redirecionamento_apos_assinar` | Redirecionamento após assinar. |
| `resposta_da_assinatura_sem_finalizacao` | Resposta da assinatura sem finalização. |
| `tela_da_assinatura_redirecionada` | Tela da assinatura redirecionada. |

### processo_concluir/erro

Emissor: [src/processo_concluir/erro.rs](../../src/processo_concluir/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `campo_nao_previsto_na_tela_de_conclusao` | Campo não previsto na tela de conclusão. |
| `charset_da_tela_de_conclusao_nao_suportado` | Charset da tela de conclusão não suportado. |
| `conclusao_nao_aplicada` | Conclusão não aplicada. |
| `conclusao_nao_publicada_na_barra` | Conclusão não publicada na barra. |
| `efeito_incerto_na_conclusao` | Efeito incerto na conclusão. |
| `redirecionamento_na_conclusao_pela_funcao` | Redirecionamento na conclusão pela função. |
| `redirecionamento_na_reabertura` | Redirecionamento na reabertura. |
| `redirecionamento_nao_reconhecido_apos_concluir` | Redirecionamento não reconhecido após concluir. |
| `redirecionamento_nao_reconhecido_apos_enviar` | Redirecionamento não reconhecido após enviar. |
| `resposta_do_concluir_sem_redirecionamento` | Resposta do concluir sem redirecionamento. |
| `resposta_do_envio_sem_redirecionamento` | Resposta do envio sem redirecionamento. |
| `situacao_do_processo_nao_lida_antes_de_concluir` | Situação do processo não lida antes de concluir. |
| `tela_da_conclusao_redirecionada` | Tela da conclusão redirecionada. |
| `tela_de_conclusao_nao_interpretada` | Tela de conclusão não interpretada. |
| `unidade_da_sessao_ausente_da_situacao` | Unidade da sessão ausente da situação. |
| `unidade_da_sessao_nao_publicada_na_abertura` | Unidade da sessão não publicada na abertura. |

### processo_enviar/erro

Emissor: [src/processo_enviar/erro.rs](../../src/processo_enviar/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `campo_nao_previsto_na_tela_de_envio` | Campo não previsto na tela de envio. |
| `charset_da_tela_de_envio_nao_suportado` | Charset da tela de envio não suportado. |
| `destino_nao_oferecido_pelo_sei` | Destino não oferecido pelo SEI. |
| `efeito_incerto_no_envio` | Efeito incerto no envio. |
| `envio_nao_aplicado` | Envio não aplicado. |
| `envio_nao_publicado_na_barra` | Envio não publicado na barra. |
| `processo_ja_aberto_no_destino` | Processo já aberto no destino. |
| `processo_nao_aberto_na_unidade_da_sessao` | Processo não aberto na unidade da sessão. |
| `resposta_do_autocompletar_fora_da_forma_observada` | Resposta do autocompletar fora da forma observada. |
| `resposta_do_autocompletar_nao_xml` | Resposta do autocompletar não XML. |
| `situacao_da_origem_diferente_da_pedida_apos_envio` | Situação da origem diferente da pedida após envio. |
| `situacao_do_processo_nao_lida_antes_de_enviar` | Situação do processo não lida antes de enviar. |
| `tela_de_envio_de_outro_processo` | Tela de envio de outro processo. |
| `tela_de_envio_nao_interpretada` | Tela de envio não interpretada. |
| `tela_de_envio_redirecionada` | Tela de envio redirecionada. |
| `varios_destinos_com_a_sigla` | Vários destinos com a sigla. |

### processo_reabrir/erro

Emissor: [src/processo_reabrir/erro.rs](../../src/processo_reabrir/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `efeito_incerto_na_reabertura` | Efeito incerto na reabertura. |
| `reabertura_nao_aplicada` | Reabertura não aplicada. |
| `reabertura_nao_publicada_na_barra` | Reabertura não publicada na barra. |
| `situacao_do_processo_nao_lida_antes_de_reabrir` | Situação do processo não lida antes de reabrir. |
| `tela_do_no_redirecionada` | Tela do no redirecionada. |
| `unidade_da_sessao_ja_na_situacao` | Unidade da sessão já na situação. |

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
| `localizacao_de_outra_unidade` | Localização de outra unidade. |
| `localizacao_fora_da_forma_observada` | Localização fora da forma observada. |
| `redirecionamento_da_troca_sem_localizacao` | Redirecionamento da troca de unidade sem localização. |
| `resposta_sem_redirecionamento` | Resposta sem redirecionamento. |
| `selecao_de_unidade_por_script_nao_publicada` | Seleção de unidade por script não publicada. |
| `sigla_de_unidade_ambigua` | Sigla de unidade ambígua. |
| `status_inesperado` | Status inesperado. |
| `tela_de_troca_de_unidade_nao_reconhecida` | Tela de troca de unidade não reconhecida. |
| `unidade_do_controle_diferente_da_pedida` | Unidade do controle diferente da pedida. |
| `unidade_marcada_diferente_da_ativa` | Unidade marcada diferente da ativa. |
| `unidade_nao_oferecida` | Unidade não oferecida. |

### unidade_trocar_cli

Emissor: [src/unidade_trocar_cli.rs](../../src/unidade_trocar_cli.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `sigla_de_unidade_vazia` | Sigla de unidade vazia. |

### visao_do_controle/definir_erro

Emissor: [src/visao_do_controle/definir_erro.rs](../../src/visao_do_controle/definir_erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `coluna_indisponivel_nesta_instalacao` | Coluna indisponível nesta instalação. |
| `configuracao_de_colunas_nao_aplicada` | Configuração de colunas não aplicada. |
| `efeito_incerto_na_configuracao_de_colunas` | Efeito incerto na configuração de colunas. |
| `marcas_do_nivel_de_detalhe_diferentes_do_alvo_apos_salvar` | Marcas do nível de detalhe diferentes do alvo após salvar. |
| `nivel_de_detalhe_devolvido_sem_sinal_de_gravacao` | Nível de detalhe devolvido sem sinal de gravação. |
| `redirecionamento_ao_salvar_nivel_de_detalhe` | Redirecionamento ao salvar nível de detalhe. |
| `resposta_do_salvar_nao_reconhecida` | Resposta do salvar não reconhecida. |


### processo_atribuir (issue #264; fatia de leitura)

Emissor: [src/processo_atribuir.rs](../../src/processo_atribuir.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `tela_da_atribuicao_fora_da_forma` | Formulário, ação assinada, identidade, lista ou controles fora da forma servida e conhecida. |
| `usuario_nao_atribuivel` | Login exato ausente da lista de usuários publicada. |
| `usuario_ambiguo_na_atribuicao` | Login aparece em mais de uma opção distinta. |
| `situacao_da_atribuicao_nao_observada` | A árvore ainda não publicou o atribuído da unidade; não presumir vazio. |
| `atribuicao_ja_aplicada` | A operação pedida já corresponde ao estado observado. |

### arvore_processo/atribuir

Emissor: src/arvore_processo/atribuir.rs. Todos os links são conferidos antes de abrir a tela.

| Subcódigo | Condição |
| --- | --- |
| `barra_da_atribuicao_fora_da_forma` | Barra do processo não interpretada. |
| `barras_da_atribuicao_repetidas` | Barras repetidas na árvore. |
| `links_de_atribuir_repetidos_na_barra` | Dois links para a mesma ação. |
| `endereco_da_atribuicao_nao_reconhecido` | Endereço não assinado ou fora da forma servida. |
| `endereco_da_atribuicao_de_outro_processo` | Identidade da ação diferente do processo aberto. |

### origem_de_treinamento

Emissor: src/origem_de_treinamento.rs. Conferido em todo `--confirmar` que grava, antes da primeira requisição.

| Subcódigo | Condição |
| --- | --- |
| `origem_fora_do_treinamento` | A origem da sessão não é instalação pública de treinamento nem a bancada `sei.test`. |
| `confirmar_sem_alteracao_de_prazo` | `processo prazo --confirmar` sem `--ate`, `--dias` ou `--excluir`. |
