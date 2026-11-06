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

## Replay público GO (05/11/2026)

No [run 42112345185](https://github.com/franklinbaldo/ies/actions/runs/42112345185), o ZIP do acervo foi conferido e o leitor Rust interpretou as seis telas da ação `unidade_selecionar_envio_processo` (três GET e três POST) sem pedidos ao SEI. No POST do passo 12, o formulário `frmUnidadeLista` aparece sem tabela `infraTable`, encerrado por `Nenhum registro encontrado.`. A estrutura agora é reconhecida como lista vazia somente para página zero e paginador de uma página; ausência de tabela sem aviso é recusada. O replay não demonstra a coleta integral de 274 páginas nem substitui as leituras live em UnB e GO.

## Replay da coleta integral GO (06/11/2026)

Identificamos no acervo um cenário mais forte que a cassette curta:
`cassettes/go/unidade-destinos-no-indice-go--85f61d670bfc.md`,
com **4 GET e 274 POST** em 278 interações reais gravadas,
**13.671 unidades**. O ZIP e a cassette são verificados pelos
hashes do card; o CI executa obrigatoriamente o leitor de 274
páginas, os filtros e a derivação de cada formulário, sem rede SEI.

A resposta do primeiro GET da lupa (passo 3) mostrou
**dois elementos `input type=submit` com `id=btnPesquisar`
e sem `name`**. Ambos são controles de submissão do formulário
`frmUnidadeLista`; como não têm nome, o navegador não transmite
um par `btnPesquisar=...`. O botão previamente exigido por
`name` era hipótese das fixtures sintéticas e causava
`formulario_da_lupa_inesperado`. A regra revisada confere o
submit servido e omite somente o parâmetro não publicado,
sem executar JavaScript ou alterar o orçamento de requests.

No [RED 42161586924](https://github.com/franklinbaldo/ies/actions/runs/42161586924)
o teste novo da derivação integral falhou na primeira página;
após a correção, [GREEN do replay integral 42162187792](https://github.com/franklinbaldo/ies/actions/runs/42162187792)
aprovou todas as 275 respostas e a união de 274 páginas.
Isso comprova a **compatibilidade offline com a cassette GO**,
não o êxito de coleta autenticada. Os ensaios live de UnB/GO
continuam com os resultados antigos e não foram repetidos.

## Replay UnB da lista de destinos (06/11/2026)

A cassette pública [UnB 57497bc1](../../cassettes/unb/unidade-destinos-no-indice-unb--57497bc11c4f.md)
registra **4 GET e 1 POST**, com **uma página e 14 unidades**.
A inspeção offline dos HTMLs revelou duas instâncias de `btnNovo`
com o **mesmo** link autorizado de cadastro (antes recusadas apenas
pela duplicação do id). Para aceitar ambas, o IES exige que cada botão,
isoladamente, publique a mesma URL de mesma origem e ação; destinos
distintos ou botão sem destino continuam recusados antes do GET.

Na lupa UnB, o botão `btnPesquisar` tem `id` mas **não tem `name`**,
e a página única omite os seletores de paginação. Enviar
`selInfraPaginacaoSuperior=0` sem que a tela o publique era uma
hipótese indevida. A derivação passa a incluir apenas os seletores
efetivamente presentes no formulário; para mais de uma página, o
seletor superior é obrigatório. A estrutura da página continua
validada antes do POST, e o total é reconciliado antes de persistir.

A sequência foi testada primeiro no [RED offline 42169131437](https://github.com/franklinbaldo/ies/actions/runs/42169131437):
quatro GET simulados, zero POST, recusa por
`campo_nao_publicado_pelo_formulario` para
`selInfraPaginacaoSuperior`. Os replays usam HTML gravado e
URL/cookie **sintéticos**, sem nova requisição ao SEI.

**Live:** UnB NOT_RUN (0 pedidos reais), GO NOT_RUN (0),
pesquisa posterior NOT_RUN (0), escritas 0, resíduos novos 0.
O replay não substitui a comprovação autenticada em treinamento.
