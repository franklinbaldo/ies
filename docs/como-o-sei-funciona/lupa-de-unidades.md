---
type: SeiObservation
title: Lupa de unidades e leitura integral dos destinos
description: Estrutura observada no oráculo Todos e limites da evidência para o IES
---

# Lupa de unidades — observação para a issue #244

Fonte: `franklinbaldo/todos`, `src/todos/sei/lupa_unidades.py`,
`src/todos/destinos.py` e `tests/test_destinos_indice_010.py`.
A evidência é **do oráculo Todos**, ainda não um replay próprio no IES.

## Ponto de entrada e efeito

A lista de grupos de envio publica `btnNovo`, que abre
`grupo_unidade_cadastrar`; a tela então publica link assinado para
`unidade_selecionar_envio_processo`. Alternativamente, o cadastro
de bloco de assinatura abre `unidade_selecionar_outras`.
O oráculo registra que abrir a tela "Novo" não grava um grupo/bloco.
Essa afirmação deve ser reconfirmada na cassette antes de uma execução
autenticada. Não basta consultar o autocomplete: em Goiás, ele limita
resultados a 50 unidades.

## Formulário e filtros publicados

A lupa usa `frmUnidadeLista` e `selOrgaoUnidade` (valor vazio
significa "Todos"; `null` é a opção sem órgão e **não** deve ser
escolhida), `txtSiglaUnidade`, `txtDescricaoUnidade`,
`hdnInfraPaginaAtual`, `selInfraPaginacaoSuperior`,
`selInfraPaginacaoInferior` e `btnPesquisar=Pesquisar`.
O `action` deve ser lido do formulário, sem compor URL assinada.
As páginas usam índice zero. O leitor deve preservar os demais
controles efetivos do formulário usando o primitivo Web existente,
substituindo apenas filtro e página; não interpretar ou executar JS.

## Forma das linhas e paginação

Tabela `infraTable` com cabeçalhos `Sigla`, `Descrição` e
`Órgão`, mais id da unidade no atributo da linha.
O oráculo de GO (SEI 5.0.3, 21/10/2026) declarou **13.671 unidades,
274 páginas de 50**. O total declarado aparece no texto
`Lista de Unidades (N registros)`; o número de páginas vem de
`selInfraPaginacaoSuperior`, e a página de
`hdnInfraPaginaAtual`.

O orçamento da **leitura integral na lupa** é um GET inicial
e um POST por página, com filtros vazios e órgão "Todos", sem
POST repetido. Conferir na resposta de cada página: identidade do
formulário, página solicitada, total e número de páginas estáveis,
IDs únicos entre páginas e soma final igual ao total declarado.
Qualquer divergência aborta sem trocar o snapshot anterior. Também
é obrigatório conferir os filtros **devolvidos em cada POST**: sigla
e descrição vazias e órgão efetivamente selecionado `Todos` (valor
vazio). Se o SEI restaurar um filtro de outra operação, uma lista
parcial pode ter total e paginação coerentes; portanto ela é recusada
antes de ser persistida. Caso de regressão derivado de
`tests/test_destinos_indice_010.py` e `tests/test_lupa_unidades_010.py`
do Todos @323eb01; o replay da cassette original ainda está pendente.

O orçamento completo do **comando** ainda precisa somar os GETs
necessários para chegar à tela de cadastro e ao link da lupa.
Isso não foi medido por esta fatia.

## Limites da evidência

O teste do Todos `test_leitura_que_nao_bate_com_o_total_nao_se_da_por_completa`
reproduz páginas repetidas e exige que o cache continue ausente.
`test_sugestoes_guardadas_para_o_envio_de_processo` demonstra
isolamento por unidade ativa e zero novas requisições nas sugestões.
Esses são cenários a traduzir para Rust, não licença para portar
a arquitetura do Todos.

**Status:** observação do oráculo PASS; replay do IES NOT_RUN;
live UnB/GO NOT_RUN. Nenhuma requisição real foi feita nesta fatia.
