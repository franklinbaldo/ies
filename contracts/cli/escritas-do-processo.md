---
type: Contract
title: Escritas do processo
description: Resumo de concluir, reabrir e enviar o processo, com os subcódigos próprios, parte do contrato da CLI
---

# Escritas do processo

Parte do [contrato da CLI](../cli.md), separada por responsabilidade.

## Concluir o processo (Refs #172)

`ies processo concluir NUP [--confirmar]` é a primeira escrita de estado do
processo ([contrato](processo-concluir.md)): prévia só com a abertura;
uma escrita pela forma que a barra publica; verificação por `aberto_em`
numa leitura nova. Subcódigos próprios: `situacao_do_processo_nao_lida_antes_de_concluir`,
`unidade_da_sessao_ausente_da_situacao`, `conclusao_nao_publicada_na_barra`,
os da barra (`formas_de_concluir_repetidas_na_barra`,
`endereco_da_conclusao_nao_reconhecido`, `endereco_da_tela_do_no_nao_reconhecido`, …),
os da tela (`campo_nao_previsto_na_tela_de_conclusao`,
`opcao_somente_concluir_nao_marcada`, `tela_de_conclusao_com_reabertura_preenchida`, …),
os da função (`funcao_de_concluir_ausente_na_tela_do_no`, …),
`conclusao_nao_aplicada` (`escrita_nao_aplicada`, saída 1) e
`nao_confirmado_na_conclusao` (`nao_confirmado`, saída 1).

## Reabrir o processo (Refs #172)

`ies processo reabrir NUP [--confirmar]` é o espelho de `processo concluir`
([contrato](processo-reabrir.md)): prévia só com a abertura; o GET da
tela do nó e o GET da URL de reabrir lida nela (a escrita); verificação
por `aberto_em` numa leitura nova. Subcódigos próprios:
`situacao_do_processo_nao_lida_antes_de_reabrir`,
`unidade_da_sessao_ja_na_situacao`, `reabertura_nao_publicada_na_barra`,
`funcoes_de_reabrir_repetidas_na_barra`, os da tela do nó
(`endereco_de_reabrir_ausente_na_tela_do_no`,
`enderecos_de_reabrir_repetidos_na_tela_do_no`, …),
`tela_do_no_redirecionada`, `redirecionamento_na_reabertura` (resposta),
`reabertura_nao_aplicada` (`escrita_nao_aplicada`, saída 1) e
`nao_confirmado_na_reabertura` (`nao_confirmado`, saída 1).

## Enviar o processo (Refs #173)

`ies processo enviar NUP --para SIGLA [--manter-aberto] [--confirmar]`
([contrato](processo-enviar.md)): prévia com a abertura, a tela
"Enviar Processo" e a consulta do autocomplete que ela declara (POST de
leitura, nenhum de escrita); um POST e verificação pelo destino e pela
unidade da sessão em `aberto_em`. Uso inválido (2):
`sigla_de_destino_invalida`, `mais_de_um_destino_nao_suportado`.
Subcódigos próprios: `situacao_do_processo_nao_lida_antes_de_enviar`,
`processo_nao_aberto_na_unidade_da_sessao`, `envio_nao_publicado_na_barra`,
`links_de_enviar_repetidos_na_barra`, os da tela
(`campo_nao_previsto_na_tela_de_envio`, `tela_de_envio_com_opcoes_preenchidas`,
`autocompletar_de_unidade_nao_declarado`, …), os da consulta
(`resposta_do_autocompletar_nao_xml`, …), `destino_nao_oferecido_pelo_sei`
(`oferecidos`), `varios_destinos_com_a_sigla` (`candidatos`),
`processo_ja_aberto_no_destino`, `envio_nao_aplicado`
(`escrita_nao_aplicada`, 1), `situacao_da_origem_diferente_da_pedida_apos_envio`
(`escrita_diferente_da_solicitada`, 1) e `nao_confirmado_no_envio`
(`nao_confirmado`, 3).
