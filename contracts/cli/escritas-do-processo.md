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
numa leitura nova. Subcódigos próprios: `SituacaoDoProcessoNaoLidaAntesDeConcluir`,
`UnidadeDaSessaoAusenteDaSituacao`, `ConclusaoNaoPublicadaNaBarra`,
os da barra (`formas_de_concluir_repetidas_na_barra`,
`endereco_da_conclusao_nao_reconhecido`, `endereco_da_tela_do_no_nao_reconhecido`, …),
os da tela (`campo_nao_previsto_na_tela_de_conclusao`,
`OpcaoSomenteConcluirNaoMarcada`, `TelaDeConclusaoComReaberturaPreenchida`, …),
os da função (`funcao_de_concluir_ausente_na_tela_do_no`, …),
`ConclusaoNaoAplicada` (saída 1) e
`NaoConfirmadoNaConclusao` (saída 1).

## Reabrir o processo (Refs #172)

`ies processo reabrir NUP [--confirmar]` é o espelho de `processo concluir`
([contrato](processo-reabrir.md)): prévia só com a abertura; o GET da
tela do nó e o GET da URL de reabrir lida nela (a escrita); verificação
por `aberto_em` numa leitura nova. Subcódigos próprios:
`SituacaoDoProcessoNaoLidaAntesDeReabrir`,
`UnidadeDaSessaoJaNaSituacao`, `ReaberturaNaoPublicadaNaBarra`,
`funcoes_de_reabrir_repetidas_na_barra`, os da tela do nó
(`endereco_de_reabrir_ausente_na_tela_do_no`,
`enderecos_de_reabrir_repetidos_na_tela_do_no`, …),
`TelaDoNoRedirecionada`, `RedirecionamentoNaReabertura` (resposta),
`ReaberturaNaoAplicada` (saída 1) e
`NaoConfirmadoNaReabertura` (saída 1).

## Enviar o processo (Refs #173)

`ies processo enviar NUP --para SIGLA [--manter-aberto] [--confirmar]`
([contrato](processo-enviar.md)): prévia com a abertura, a tela
"Enviar Processo" e a consulta do autocomplete que ela declara (POST de
leitura, nenhum de escrita); um POST e verificação pelo destino e pela
unidade da sessão em `aberto_em`. Uso inválido (2):
`sigla_de_destino_invalida`, `mais_de_um_destino_nao_suportado`.
Subcódigos próprios: `SituacaoDoProcessoNaoLidaAntesDeEnviar`,
`ProcessoNaoAbertoNaUnidadeDaSessao`, `EnvioNaoPublicadoNaBarra`,
`links_de_enviar_repetidos_na_barra`, os da tela
(`campo_nao_previsto_na_tela_de_envio`, `TelaDeEnvioComOpcoesPreenchidas`,
`AutocompletarDeUnidadeNaoDeclarado`, …), os da consulta
(`RespostaDoAutocompletarNaoXml`, …), `DestinoNaoOferecidoPeloSei`
(`oferecidos`), `VariosDestinosComASigla` (`candidatos`),
`ProcessoJaAbertoNoDestino`, `EnvioNaoAplicado`
(1), `SituacaoDaOrigemDiferenteDaPedidaAposEnvio`
(1) e `NaoConfirmadoNoEnvio`
(1).
