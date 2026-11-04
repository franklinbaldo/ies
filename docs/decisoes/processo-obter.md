---
type: ImplementationDecision
title: Ler processo pela caixa (resumo), pela árvore com índice local e pelo histórico
operation: processo.obter
implementation: partial
status: adopted-for-scope
scope: resumo na caixa sem abrir o processo; leitura do primeiro nível da árvore abrindo o processo pelo caminho mais barato do índice local (Refs #59, fatia 1); andamentos pelo link do histórico publicado na árvore, visão servida ou completa (fatia 2); pastas abertas por Abrir todas as Pastas e conferidas pela identidade dos protocolos (fatia 3); colheita do Controle no índice e passo 1b pelo link procedimento_trabalhar colhido (fatia 4); processos anexados lidos do nó PROCESSO_ANEXADO da mesma árvore, sem os documentos deles, e o pai do processo anexado lido do link do nó raiz; propriedades pelo formulário procedimento_alterar (ou procedimento_consultar) publicado na barra do nó do processo, sem envio; marcadores pela tela andamento_marcador_gerenciar da barra, sempre lista (apêndice)
selected_strategy: web-indice-pesquisa-rapida-arvore
alternatives: web-indice-pesquisa-rapida-arvore,web-pesquisa-avancada-arvore,web-caixa-somente,rest-wssei,web-historico-link-da-arvore,web-historico-url-montada,web-abrir-todas-as-pastas,web-paginar-pasta-por-post,web-colheita-do-controle,web-get-extra-para-o-indice,web-anexados-do-no-da-arvore,web-abrir-cada-anexado,web-vinculo-recusa-fechada,web-vinculo-nulo-com-motivo,web-propriedades-da-barra,web-propriedades-todas-as-telas,web-propriedades-listas-ocultas,web-marcadores-da-barra,web-marcadores-da-celula-do-controle
evidence_level: capturas-privadas-estrutura-cassettes-go-e-testes-sinteticos
live_validation: pass-unb-go-pen1-fatias-1-e-2
---

# Ler processo: resumo na caixa e árvore com índice local

## Alternativas consideradas

- **web-indice-pesquisa-rapida-arvore** (escolhida para a leitura pela
  árvore): link da árvore guardado → entrada por id → pesquisa rápida
  guardada → Controle (linha do NUP exato ou pesquisa rápida do
  formulário), depois tela do processo e árvore.
- **web-pesquisa-avancada-arvore:** o fluxo de `processo buscar`
  (Controle → tela de pesquisa → POST) e o 302 da pesquisa avançada.
- **web-caixa-somente:** abrir só processos com linha no Controle.
- **rest-wssei:** módulo REST opcional, ausente em muitas instalações
  ([ADR 0001](../arquitetura/0001-rotas-por-evidencia.md)).

O `--resumo` continua a decisão anterior: um GET da caixa, sem abrir o
processo nem enviar POST.

## Comparação

- **Pesquisa avançada rejeitada.** Medição do agente pai em 03/11/2026:
  em GO e na PEN1 a pesquisa avançada não achou NUPs que a pesquisa
  rápida, com o NUP exato, abriu (302 para `procedimento_trabalhar`
  assinado, também na UnB). Ela ainda custa uma requisição a mais
  (Controle, tela, POST) e devolve listas que o comando teria de
  interpretar.
- **Só a caixa** não abre processos fora da unidade e faria a ausência
  parecer inexistência.
- **Pesquisa rápida** é um POST de um campo para o `action` assinado do
  `form#frmProtocoloPesquisaRapida`, servido no Controle (UnB, GO, PEN1,
  PEN3, PEN4, UFRJ) e na tela do processo (UnB, GO). O 302 observado nas
  cassettes de GO tem `acao_origem=protocolo_pesquisa_rapida` e o id em
  `id_protocolo`; NUP inexistente respondeu 200 com a tela de pesquisa
  (que o IES não lê).
- **Índice local** ([ADR 0003](../arquitetura/0003-indice-local.md)):
  dentro da mesma sessão o link da árvore é reaberto com um GET; com o id
  conhecido, a entrada sem assinatura medida na UnB custa três GET; a
  recusa dela derruba a sessão de todos, por isso é registrada e o
  comando para pedindo `ies login`.

Orçamento por caminho: link da árvore **1**; entrada por id **3**;
pesquisa rápida guardada **3**; linha do Controle **3**; pesquisa rápida
do Controle **4**. Um palpite que falha soma suas requisições.

## Decisão

`ies processo NUP` sem `--resumo` segue a ordem aprovada pelo
proprietário, definida no
[contrato da árvore](../../contracts/cli/processo-obter-arvore.md), lê
a árvore com um leitor de literais JS escrito à mão (sem regex, sem
executar JavaScript) e publica só o observado: identidade do nó
`PROCESSO`, `documentos` do primeiro nível sem a dica (dado pessoal em
e-mails), `unidade` da tela que a mostrou (`null` no link da árvore) e
`nao_lidos`. O índice aprende NUP ↔ id e os links da tela do processo, da
árvore e da pesquisa rápida; palpite errado é esquecido e o comando segue.

**Efeito declarado.** O GET da árvore marcou como visualizado, em GO, um
processo recebido não visualizado; o de `procedimento_trabalhar` sozinho
não. O IES não restaura esse estado. Lançamento de andamento: não medido.

## Evidências verificáveis

- [Contrato da operação](../../contracts/cli/processo-obter.md)
- [Matriz por instalação](../../contracts/evidencias/processo-obter.md)
- [Árvore do processo](../como-o-sei-funciona/arvore-do-processo.md)
- [Testes de superfície](../../tests/cli_contract.rs)
- Testes do fluxo com transporte roteirizado e índice temporário em
  `src/processo_obter/tests_fluxo.rs`, `tests_indice.rs`,
  `tests_pastas.rs` e `tests_colheita.rs` (passo 1b); colheita em
  `src/colheita_do_controle/tests.rs`; leitor da árvore em
  `src/arvore_processo/tests*.rs`.
- [Contrato das pastas](../../contracts/cli/processo-obter-pastas.md)
- [Contrato das propriedades](../../contracts/cli/processo-obter-propriedades.md);
  testes em `src/processo_obter/tests_propriedades.rs`,
  `src/propriedades_processo/tests_tela.rs` e
  `src/arvore_processo/tests_barra.rs`
- Capturas privadas `0600` (UnB e GO) lidas só pela estrutura: tela do
  processo, árvore e o formulário rápido e as linhas do Controle de seis
  instalações passaram pelos leitores; nenhum valor foi copiado.

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
`andamentos: null` com motivo, não como falha. O leitor de linhas de
`--historico-privado` (`src/historico.rs`) leu as páginas reais sem
mudança.

**Orçamento.** Padrão +1 GET; `--andamentos-completos` +1 GET +1 POST
(o POST só se o GET serviu `R`); `--sem-andamentos` +0. Repetição quente
no caminho 1: árvore e histórico guardado, **2**. Tabela por caminho no
[contrato dos andamentos](../../contracts/cli/processo-obter-andamentos.md).

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
([ADR 0003](../arquitetura/0003-indice-local.md)); `ies processo NUP`
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

## Processos anexados

**Alternativas.** (a) **web-anexados-do-no-da-arvore** (escolhida): o nó
`PROCESSO_ANEXADO` da árvore já lida dá número, id e tipo de cada
anexado; o índice aprende o par NUP ↔ id; os documentos do anexado não
são lidos e a saída diz isso (`documentos_dos_anexados` em `nao_lidos`).
(b) **web-abrir-cada-anexado**: seguir o a3 de cada anexado para listar
os documentos dele; rejeitada: uma requisição por anexado, efeito do GET
não medido e forma da resposta nunca observada. (c) Continuar recusando
a árvore inteira: o comando falhava num processo real.

**Evidência.** Captura privada da UnB 4.0.9 (03/11/2026): um
`PROCESSO_ANEXADO` de 16 argumentos com pai igual ao processo, a15 igual
a a5 e nenhum nó filho dele ([evidência](../../contracts/evidencias/processo-obter.md)).

**Orçamento.** 0 requisições a mais em todos os caminhos.

**Trade-offs.** Quem quer os documentos do anexado roda
`ies processo <NUP do anexado>`, que já sai pelo índice. Filho de
anexado, nunca observado, é recusa (`no_com_pai_no_processo_anexado`):
falha fechada até haver captura.

**Pai do processo aberto.** Aberto um anexado, o `.html` da raiz traz o
link assinado do pai (UnB 4.0.9, 03/11/2026). O IES publica `anexado` e
`anexado_a` e aprende o par e o link `procedimento_trabalhar` do pai, sem
requisição: depois, `ies processo <NUP do pai>` custa 2 GET (passo 1b).
Link candidato fora da forma ou repetido: (a) **web-vinculo-recusa-fechada**
(escolhida), recusa tipada; (b) **web-vinculo-nulo-com-motivo**, publicar
a árvore com `anexado_a: null` e um motivo. Escolhida (a): com um link
candidato malformado, `anexado: false` seria afirmação falsa e (b)
exigiria um terceiro estado; o leitor da árvore inteira já falha fechado,
e a recusa diz o fato.

## Propriedades

**Alternativas.** (a) **web-propriedades-da-barra** (escolhida): um GET
do `procedimento_alterar` assinado que a barra do nó `PROCESSO` publica
(ou de `procedimento_consultar`, quando alterar não é oferecido), lendo os
controles servidos de `form#frmProcedimentoCadastro`, sem envio. (b)
**web-propriedades-todas-as-telas**: as quatro telas do Todos (metadados,
anotação, atribuição, relacionamentos); rejeitada nesta fatia, 4 GETs e
três telas nunca capturadas pelo IES. (c)
**web-propriedades-listas-ocultas**: ler `hdnAssuntos` e
`hdnInteressadosProcedimento`; rejeitada: vêm vazias no HTML servido (o
`infraLupaSelect` as monta no navegador), então o `select` é a única
fonte; lista servida preenchida é recusa até ser observada.

**Evidência.** Captura privada `0600` da UnB 4.0.9 (04/11/2026), só
estrutura ([observação](../como-o-sei-funciona/propriedades-do-processo.md)):
a tela passou pelo leitor (tipo marcado, 1 assunto com código, 0
interessados, observação vazia, `publico`, sem hipótese, data); as seis
árvores capturadas publicaram o link (quatro alterar, duas consultar,
processos abertos só noutra unidade); teste local temporário, apagado.
Oráculo: `todos/propriedades.py` lê os mesmos controles e troca alterar
por consultar com o processo fechado (UFRJ 4.1).

**Forma publicada.** `assuntos` saem como `{codigo, descricao}` porque o
texto servido é `<código> - <descrição>` com código de dígitos e pontos;
fora disso `codigo` é `null` e o texto inteiro é a descrição, sem
palpite. `tipo` e `hipotese_legal` são a `option` marcada (nenhuma:
`null`); `hdnIdTipoProcedimento` confere o tipo. Ids de assunto e de
contato não são publicados. `fonte` diz qual tela foi lida.

**Orçamento.** +1 GET; `--sem-propriedades` +0; barra sem o link +0.
Repetição quente no caminho 1: árvore, histórico e propriedades
guardados, **3** (**2** com `--sem-andamentos`, **1** com os dois
`--sem-*`). Guardado só da `acao` que a barra recém-lida oferece:
diferente dos andamentos, a barra decide qual tela vale (alterar some
quando o processo fecha na unidade).

**Efeito.** GET de formulário de edição, declarado leitura (o Todos o
usa como leitura); nunca POST, nunca `btnSalvar`. Nada a restaurar.

**Trade-offs.** Ler o formulário de edição em vez de uma tela de
consulta custa o mesmo GET e publica o que o SEI gravou; recusar formas
não observadas (listas ocultas, nível sem marca) pode derrubar o comando
numa instalação nova, e por isso `--sem-propriedades` existe.

## Limitações e revisão

Propriedades: validação live **PASS** na UnB, PEN1 NOT_RUN por `DOCUMENTO_MOVIDO` ([relatório](../validacao-live/2026-11-04-processo-obter-propriedades.md)).
Validação live: **PASS** na UnB, em GO e na PEN1 nos caminhos 1, 2, 3 e 4a, e na UnB também no 4b ([relatório](../validacao-live/2026-11-03-processo-obter-arvore.md)); recusa da entrada por id NOT_RUN ao vivo. Marcadores:
[apêndice](processo-obter/marcadores.md), validação live NOT_RUN; pastas: validação live
**PASS** na UnB, frio, quente e sessão nova, com o servidor sem lembrar as pastas abertas ([relatório](../validacao-live/2026-11-03-processo-obter-pastas.md)).
Andamentos: validação live **PASS** na UnB e na PEN1 (frio, quente,
completo, `--sem-andamentos`) e em GO (frio), com a visão `P` **lembrada**
pelo servidor ([relatório](../validacao-live/2026-11-03-processo-obter-andamentos.md)).
Colheita e passo 1b: validação live **PASS** na UnB e na PEN1 ([relatório](../validacao-live/2026-11-03-colheita-do-controle.md)).
Revisar se a legenda trouxer paginação ou se outra instalação publicar o
link noutra forma. Revisar se a entrada por id responder outra forma, se o 302 da
pesquisa rápida mudar de chave do id fora de GO, ou se a árvore publicar
outro construtor.
