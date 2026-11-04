---
type: ImplementationDecision
title: Ler a versão N pela lista da barra do documento e a versão atual pelo rodapé servido
operation: documento.obter
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de documento.obter (Refs #88); ies documento DOC --versao N lê a versão N pela lista documento_versao_listar que a barra do DOCUMENTO publica na árvore; versao_atual sai do rodapé Criado por …, versão N do conteúdo lido ou da lista lida; --saida sem --sem-copia prepara DOC.vN.html com base e manifesto quando a barra declara editarConteudo('N')
selected_strategy: web-lista-da-barra-e-rodape
alternatives: web-lista-da-barra-e-rodape,web-lista-sempre,web-url-montada-por-versao,web-editor-montar,web-sem-versao-atual
evidence_level: acervo-publico-sete-instalacoes-testes-sinteticos-e-live-pen4
live_validation: pass-pen4-5-0-4
---

# Versões de documento

Apêndice da [decisão de `documento.obter`](../documento-obter.md);
contrato em [versões e cópia de trabalho](../../../contracts/cli/documento-obter-versoes.md).

## Alternativas consideradas

- **web-lista-da-barra-e-rodape** (escolhida): a versão do conteúdo lido
  vem do rodapé que o próprio `documento_visualizar` serve; a lista
  (`documento_versao_listar`, link assinado da barra do nó) só é pedida
  por `--versao N` ou pela cópia quando falta o rodapé.
- **web-lista-sempre:** ler a lista em toda leitura para publicar
  `versao_atual` (+1 GET por documento).
- **web-url-montada-por-versao:** montar `documento_visualizar&versao=N`
  pelo id.
- **web-editor-montar:** abrir o editor para saber a versão e preparar a
  cópia.
- **web-sem-versao-atual:** manter `versao_atual: null` sempre.

## Comparação

- **Custo.** O proprietário decidiu (04/11/2026) que `versao_atual` é
  `null` salvo quando pedida ou já servida na tela lida. O rodapé
  `Criado por U, versão N por U em DATA.` (`div unselectable="on"`) está
  na tela do conteúdo: 0 GET. `web-lista-sempre` custaria +1 GET em todo
  documento; `web-sem-versao-atual` jogaria fora um dado servido.
- **Segurança da sessão.** A ADR 0003 registra que `documento_versao_listar`
  e `documento_visualizar` sem assinatura derrubam a sessão: URL montada
  rejeitada. A barra publica o link assinado (conferido por `id_documento`
  e `id_procedimento`) e a lista publica um `documento_visualizar`
  assinado por linha.
- **Editor.** `editor_montar` é escrita em potencial e fica com #89; a
  barra já declara `editarConteudo('N')` (não assinado na montagem) ou
  `'S'` (assinado), sem abrir nada.
- **`--versao N` sem o conteúdo atual.** O texto publicado é o da versão
  N; ler também o atual seria um GET sem uso. Custo: passos 1 a 4 + 2.

## Decisão

Adotada **web-lista-da-barra-e-rodape**. Orçamentos exatos nos testes
(`G` = 3 com o `action` guardado, 4 sem): leitura G + 1 (com
`versao_atual` do rodapé), `--versao N` G + 2, cópia G + 1 com rodapé e
G + 2 sem ele. O leitor da lista (#91, #94, #115) passou a aceitar a
forma servida: `id_documento` duas vezes com o mesmo valor e a âncora
`#ID-N` do "Recuperar Versão" (link no `onclick`, nunca seguido). Com
`--versao N`, um rodapé de outra versão é recusa
(`versao_servida_diferente_da_pedida`).

## Evidências verificáveis

- Acervo público (replay `src/processo_obter/acervo_versoes.rs`,
  ignorado no CI, 280 cassettes): barras de documento lidas sem recusa
  nas sete instalações (lista com `'N'`, com `'S'`, sem edição, sem
  lista); as sete listas gravadas (`liberacoes-e-versoes-leitura-*`)
  lidas com 2 versões; 91 conteúdos HTML (UnB, PEN1 a PEN4) com o
  rodapé na forma exata.
- Testes sintéticos: `arvore_processo::tests_barra_documento`,
  `documento_versoes::testes`, `rodape_do_documento::testes`,
  `processo_obter::tests_documento_versoes` (sequências exatas),
  `documento_versao_cli::testes`, `copia_documento::testes` e
  [cli_contract](../../../tests/cli_contract.rs).
- RED: 15 testes falhando com shims (barra, lista, fluxo, cópia, CLI);
  depois, 5 do rodapé antes do leitor.
- [Ensaio live na PEN4 5.0.4](../../validacao-live/2026-11-04-documento-versoes.md).

## Limitações e revisão

O `'N'` de `editarConteudo` é a declaração da árvore na montagem; o JS
do SEI ainda confere a assinatura por AJAX antes de editar, e o `salvar`
remoto (#89) terá de conferir de novo. A cópia é o HTML do
`documento_visualizar`, não o do editor: as seções editáveis só se
conhecem ao observar o editor (#89). Gravada em UTF-8, embora o `meta`
servido declare `iso-8859-1`. UFRJ e GO não gravaram
`documento_visualizar`: o rodapé não foi visto lá (sem ele, a cópia lê a
lista). Não observados: lista paginada, documento em pasta com barra
concatenada não vazia, efeito de visita do GET da lista. Rever se o
rodapé mudar de forma ou se a barra publicar a edição de outro jeito.
