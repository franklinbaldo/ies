---
type: Guide
title: Decisão de processo obter — andamentos, pastas e colheita
description: Fatias 2 a 4 da decisão de processo obter, com andamentos, pastas e colheita do Controle
---

# Decisão de processo obter — andamentos, pastas e colheita

Parte do [decisão de processo obter](../processo-obter.md), separada por responsabilidade.

## Andamentos (fatia 2)

**Alternativas.** (a) **web-historico-link-da-arvore** (escolhida): o link
assinado `procedimento_consultar_historico` que a árvore publica, nas duas
formas observadas (função servida na UnB, `onclick` com a URL em GO), e o
POST `verHistorico('P')` do formulário servido para a visão completa.
(b) **web-historico-url-montada**: montar a URL do histórico pelo id;
rejeitada, porque sem `infra_hash` a instalação recusa e pode derrubar a
sessão (como na entrada por id), e o IES só monta a entrada por id. (c)
REST: fora da prioridade (ADR 0001). (d) Ler os andamentos da tela do
processo: a tela não os publica.

**Evidência.** Capturas privadas `0600` (UnB 4.0.9 e GO 5.0.3, 03/11/2026)
lidas só pela estrutura passaram pelos leitores: as duas formas do link,
as páginas `R` (UnB) e `P` (GO, 49 e 17 linhas), a regra servida de
`verHistorico` e a derivação do POST pelo primitivo de formulário (dez
ocultos, nenhum botão). Medições do agente pai: a visão `P` de 14
processos de GO veio inteira até 75 linhas, sem paginador; a ablação na
UnB mostrou que `hdnInfraNroItens` e `hdnInfraPaginaAtual` não paginam o
histórico. Duas árvores de GO não publicaram o link: isso sai como
`andamentos: null` com motivo, não como falha. O leitor de linhas do
histórico (`src/historico.rs`, o do antigo `ies replay`) leu as páginas reais sem
mudança.

**Orçamento.** Padrão +1 GET; `--andamentos-completos` +1 GET +1 POST
(o POST só se o GET serviu `R`); `--sem-andamentos` +0. Repetição quente
no caminho 1: árvore e histórico guardado, **2**. Tabela por caminho no
[contrato dos andamentos](../../../contracts/cli/processo-obter-andamentos.md).

**Efeitos.** O GET do histórico é leitura (lançamento de andamento não
medido). O POST `P` é declarado `PaginaOuFiltroLembrado`: não se mediu
se a visão fica lembrada. A visão não é restaurada (vários agentes na
mesma conta se desfariam). O botão `btnAtualizarAndamento` (escrita) nunca
participa. Falha HTTP no histórico (503 visto em GO) encerra o comando com
o erro de transporte, sem retry; o 403 do WAF de GO é
`bloqueio_de_seguranca_da_instalacao`.

**Trade-offs.** Ler sempre a visão servida custa um GET a mais por
comando, mas publica o que a pessoa vê no SEI; o completo exige uma
mudança de estado de sessão declarada. Legenda com faixa de paginação é
recusa até ser observada.

## Pastas (fatia 3)

**Alternativas.** (a) **web-abrir-todas-as-pastas** (escolhida): o GET da
ação assinada `ABRIR_PASTAS` que a árvore publica, só quando há `AGUARDE`,
e a releitura da árvore inteira. (b) **web-paginar-pasta-por-post**: um
POST de `frmArvore` para `Pastas[k]['link']` por pasta fechada, como o
navegador; custa uma requisição por pasta (7 contra 1 na cassette de 8
pastas) e ficou fora desta fatia, porque as duas cassettes publicam a ação
de abrir. (c) Um POST com os protocolos de várias pastas: o Todos mediu a
recusa "Conjunto de protocolos inválido". (d) Publicar só a pasta aberta:
seria lista parcial.

**Evidência.** Cassettes públicas da UnB `processo-em-pastas-unb`
(SHA-256 `0bc73a70…a369`, interações 8 e 9: 8 pastas, 7 `AGUARDE`, 17 e
157 documentos) e `processo-pasta-mista-unb` (SHA-256 `dffe2d93…dd42`,
interações 411 a 487: uma pasta fechada e uma aberta; a árvore simples
volta fechada depois do GET de abrir). O Todos, oráculo de comportamento,
confere as pastas pelo `id_rel_protocolo_protocolo` da barra; a mesma
regra deu 17/17 e 157/157 nas duas leituras.

**Integridade.** Por identidade quando a barra a publica: cada protocolo
declarado exatamente uma vez entre os documentos da pasta, nenhum fora da
declaração. O Todos exige também a ordem; o IES publica a ordem servida e
não a usa como prova. **Barras vazias** (decisão do proprietário,
03/11/2026, opção B): ao vivo na UnB, dois processos com pastas (157
documentos em 8 pastas e um de 2 pastas), abertos só em outra unidade,
abriram inteiros por `ABRIR_PASTAS`, com contagens iguais às declaradas
(7×20 + 17), mas 150 barras vazias (`Nos[i].acoes = '';`). Uma primeira
versão da B (regra por pasta, tudo ou nada) falhou na revalidação ao
vivo: a identidade varia **por documento** (no processo A, PASTA6 com 18
barras vazias e 2 com links sem `id_rel_protocolo_protocolo`, PASTA8 com
12 e 5; no B, PASTA1 com 19 identidades e 1 vazia). Regra adotada: cada
identidade achada está na declaração e aparece uma vez; os declarados sem
documento identificado são tantos quantos os documentos sem identidade
(`contagem_da_pasta_diferente_da_declarada`); ids únicos na árvore;
nenhum `AGUARDE`. Cada pasta diz `por_identidade`, `mista` ou
`por_contagem`, com as duas quantidades; o topo, a mais fraca, com o
motivo `documentos_sem_identidade_na_barra`. Nas duas árvores reais: A
`por_contagem` (8 pastas, 0 identidades), B `mista` (PASTA1 19+1, PASTA2
2+0). **Opção A rejeitada:** manter só a identidade recusaria esses processos inteiros
(`documento_da_pasta_sem_identidade_na_barra`) e o usuário não receberia
nada, embora a árvore servida seja coerente; a B entrega a lista e diz
qual conferência foi feita, sem alegar identidade onde ela não existe. A
regra é a barra observada, nunca o texto "aberto somente na
unidade" (mensagem é apresentação).

**Orçamento e índice.** Sem `AGUARDE`, +0; com `AGUARDE`, +1 GET. A URL
de abrir é guardada com o objeto `<id>:abrir_pastas` e tentada primeiro no
caminho 1: a repetição quente custa **1**. Efeito declarado
`PaginaOuFiltroLembrado` (não medido pelo IES; nas cassettes a leitura
seguinte voltou fechada). Nunca `FECHAR_PASTAS`, sem restauração.

**Trade-offs.** Abrir todas custa ~1 MB num GET em vez de vários POSTs
pequenos; a contagem prova quantidade e unicidade, não que cada documento
é o protocolo declarado, e por isso é publicada como verificação mais
fraca.

## Colheita do Controle e passo 1b (fatia 4)

**Alternativas.** (a) **web-colheita-do-controle** (escolhida): todo
comando que já lê o Controle grava no índice, de cada linha válida, o par
NUP ↔ id e o `href` assinado de `procedimento_trabalhar`, mais o link da
pesquisa e o `action` da pesquisa rápida da mesma página
([ADR 0003](../../arquitetura/0003-indice-local.md)); `ies processo NUP`
ganha o passo 1b, que abre esse link. (b)
**web-get-extra-para-o-indice**: um GET do Controle (ou de páginas
seguintes) só para alimentar o índice; rejeitada, porque o princípio é
nunca pedir algo só para o índice. (c) Sem colheita: o processo visto na
caixa custa a entrada por id (3 GET, e a recusa derruba a sessão) ou o
caminho 4 inteiro.

**Evidência.** O `href` de `tr#P{id}` nas visões R e D, já lido pelo
caminho 4a (UnB, GO, PEN1, PEN3, PEN4, UFRJ), é o mesmo
`procedimento_trabalhar` assinado que o caminho 4a abre: o 1b é o 4a sem
o GET do Controle. O hash é reutilizável dentro da sessão (ADR 0003).

**Orçamento.** 1b: **2 GET** até a árvore; com andamentos, +1 GET
(**3**). Na colheita, **0**: nenhuma requisição, nenhuma página a mais, a
saída dos comandos não muda. O passo vem depois do 1 (link da árvore, 1
GET) e antes da entrada por id, que custa 3 e arrisca a sessão.

**Efeito declarado.** O GET de `procedimento_trabalhar` é o mesmo do
caminho 4a (não marcou visualizado em GO; a árvore marcou).

**Trade-offs.** O Controle publica dezenas de linhas por página: a
colheita grava dezenas de links de vida curta, que só valem nesta
geração e unidade e são conferidos de novo a cada uso. Link velho custa
1 GET antes do passo seguinte.
