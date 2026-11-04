---
type: ImplementationDecision
title: Pesquisa de processo pela Web e em HTML privado
operation: processo.buscar
implementation: partial
status: adopted-for-scope
scope: pesquisa por protocolo pela sessão de ies login (página observada ou --pagina N) e seleção exata de NUP em resposta capturada
selected_strategy: web-post-authorized
alternatives: web-post-authorized,web-captured-offline,rest-wssei,soap-official
evidence_level: ensaio-live-unb-go-pen1-pen3-e-replay-do-acervo
live_validation: pass-unb-go-pen3-pen1-sem-indice
---

# Decisão de implementação de processo.buscar

## Alternativas consideradas

- **web-post-authorized:** `ies processo buscar TERMO [--pagina N]`: GET do
  Controle, link `protocolo_pesquisar` publicado, GET da tela, POST da
  pesquisa e, na página N, um POST de `navegar()`, com a sessão gravada por
  `ies login`. Orçamento: 3 requisições na página 1 e 4 na página N.
- **web-captured-offline:** resposta HTML já obtida pelo operador, parser
  estrito e zero requisições (`--resultado-pesquisa-file`, aposentado em
  04/11/2026 por decisão do proprietário; a leitura offline fica em
  `ies replay --resultado-pesquisa`).
- **rest-wssei:** módulo opcional e token não comprovados por instalação.
- **soap-official:** autenticação e equivalência desconhecidas.

## Comparação

A rota Web foi ensaiada pelo proprietário na UnB 4.0.9 (03/11/2026): o
POST derivado como abaixo foi **aceito** (302 para NUP exato, 200 com
`pesquisaSemResultado` para inexistente, 200 com tabela e barra para
parcial), com efeito `PaginaOuFiltroLembrado`. Custa 3 ou 4 requisições e
altera a página/filtro lembrados, mas responde para a conta e a unidade
reais. A offline não acessa o SEI e não comprova presença atual. REST e
SOAP continuam sem disponibilidade comprovada.

## Decisão

Adotar a rota Web para o comando `buscar`, mantendo a fatia offline:

- o GET do Controle valida origem antes do cookie; a unidade vem do
  Controle reconhecido (R ou D) e confere `--unidade`;
- só o link do menu com `acao=protocolo_pesquisar` da **mesma resposta**
  é seguido: relativo `controlador.php?`, uma `acao`, um `infra_hash` de 64
  hex, chaves canônicas, mesma instalação; ausente, repetido, absoluto ou
  fora da forma é recusa sem rede;
- o POST usa `PESQUISAR_PROTOCOLO` (const da operação, sem despachante),
  agora com `ResultadoDoEnvio::UmDosMarcadores` (`table.pesquisaResultado`
  ou `div.pesquisaSemResultado`); a validação da Infra continua recusa
  (`validacao_recusada_pelo_sei`) e a falta de marcador vira
  `resposta_da_pesquisa_nao_reconhecida`;
- 302 → `processo_unico` só com o `id`; o número formatado não é inventado
  (abrir o processo é #59);
- lista → `total_declarado` e `faixa_observada` da barra "Exibindo I - F
  de N" ou "N resultados" (lista numa página só), separador `.` só entre
  grupos de 3 dígitos;
- `sem_resultado` é saída 0 e **não** `processo_inexistente`;
- `--pagina N` usa `PAGINAR_PESQUISA_DE_PROTOCOLO` (`SubmitPorScript`,
  `hdnInicio` reescrito) derivado da tela de resultado, depois de conferir
  as linhas literais de `navegar()`, os links `navegar('K')` e o termo
  servido; N fora do intervalo falha antes do POST; sem restauração.

As linhas reais da UnB têm `a.arvore` com o mesmo `href` do
`a.protocoloNormal`, e este repete `class` idêntico. O parser aceita só essa
forma, pelo tokenizador de `src/html_atributos`
(`valor_com_repeticao_identica`); valores diferentes continuam recusados.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-buscar.md)
- [Matriz](../../contracts/evidencias/processo-buscar.md)
- [Observação da pesquisa](../como-o-sei-funciona/pesquisa-de-protocolo.md)
- [Superfície da CLI](../../tests/processo_buscar_web.rs)
- Testes unitários: [fluxo](../../src/pesquisa_web/tests_fluxo.rs) (sequência
  exata de requisições), [resposta](../../src/pesquisa_web/tests_resposta.rs),
  [paginação](../../src/pesquisa_web/tests_paginacao.rs),
  [link](../../src/pesquisa_web/tests_link.rs),
  [linhas da UnB](../../src/pesquisa_web/tests_arvore.rs) e
  [JSON](../../src/processo_buscar_cli/tests.rs)
- [Parser da resposta](../../src/pesquisa_web.rs)
- [Política de acesso](../interfaces/acesso-por-instalacao.md)
- [Replay do acervo](processo-buscar/replay-do-acervo.md) (apêndice,
  04/11/2026): as peças do comando sobre as 52 cassettes com a Pesquisa
  nas sete instalações, e a linha com acerto em documento de Goiás.

Ensaio estrutural local (03/11/2026, teste temporário não versionado, sobre
as capturas privadas `0600`): a resposta parcial virou lista de 10 linhas
com total e faixa 1–10; o plano da página 2 deu `hdnInicio=10` e faixa
11–20; o POST de página derivado da tela real não leva `sbmPesquisar`; a
resposta inexistente virou `sem_resultado`; o link do menu foi achado nos
Controles da UnB, PEN1 e GO (**PASS**).

## Regras do `onsubmit` da derivação

`derivar_pesquisa_de_protocolo` (em
`src/pesquisa_web/formulario_pesquisa.rs`) prepara o POST da
[decisão 0002](../arquitetura/0002-submeter-formulario-observado.md),
sem rede. As regras foram escritas à mão a partir do JavaScript servido no
GET de `protocolo_pesquisar` (interações 1, 3, 5 e 9) das cassettes
`processo-busca-*` de UnB, PEN1, UFRJ e Goiás. O IES não executa nem
interpreta JavaScript.

- `onSubmitForm()` exige `#optProcessos` ou `#optDocumentos` marcado. A
  tela vem com `optDocumentos` (`D`) marcado nas quatro instalações, e `D`
  exige ainda `chkSinDocumentosGerados` ou `chkSinDocumentosRecebidos`,
  checkboxes que a derivação não envia. Para buscar processo, a operação
  envia o valor servido de `#optProcessos` (nome `rdoPesquisarEm`,
  observado `P`), como o clique no rádio. Rádio ausente, repetido, sem
  `value` ou com outro nome são recusas próprias.
- Com `P`, `tratarCamposDocumento()` esvazia `hdnIdAssinante`, entre
  outros campos que a derivação não envia. Como os ocultos saem como
  servidos, `hdnIdAssinante` não vazio é recusado.
- `onSubmitForm()` exige órgão em `selOrgaoPesquisa[]` (`multipleSelect`,
  `getSelects`). A operação envia as opções com `selected` servidas, na
  ordem do documento, e não inventa órgão. Seleção servida: 1 opção na
  UnB, 2 na PEN1 e na UFRJ e 62 em Goiás. `select` ausente ou repetido,
  sem `multiple`, sem seleção ou com opção selecionada sem `value` são
  recusas próprias.
- `navegar(inicio)` só reescreve `hdnInicio` na paginação; `sbmPesquisar`
  não tem `onclick`. O valor servido foi `0` nos 16 GETs; outro valor
  pediria página posterior e é recusado.
- Contato, datas e, na UFRJ e em Goiás, faixa de valores só são conferidos
  quando preenchidos; a derivação não os envia. `onSubmitForm()` é igual
  na UnB e na PEN1, e na UFRJ e em Goiás tem essas conferências a mais. A
  operação codifica a **interseção**.

Ensaio offline: a derivação passou nos 16 formulários reais (4 GETs por
instalação), com 1, 2, 2 e 62 órgãos enviados (**PASS**, teste local
temporário com arquivos `0600` fora do repositório, não versionado).

O `select` real tem `multiple` sem valor antes de `id`. O `tl` 0.7.8
perde o primeiro caractere do atributo seguinte a um booleano sem valor
(`id` vira `d`; `multiple disabled` vira `isabled`). **Corrigido**: todo
atributo lido fora dos testes passa por `src/html_atributos.rs`, que
tokeniza a fonte da tag de abertura com os estados WHATWG; `selected`,
`multiple` e `disabled` são detectados em qualquer forma de valor. O
órgão continua localizado pelo `name`, que é o nome serializado no POST
e, por isso, a regra mais precisa. Testes: `tests_formulario_pesquisa.rs`
(booleano antes de cada atributo e `multiple disabled` bruto).

Como no navegador, uma opção de órgão `selected` mas desabilitada, por
`disabled` próprio ou de um `optgroup` ancestral, **não** é enviada;
nenhuma outra é posta no lugar. Se todas as selecionadas estiverem
desabilitadas, a recusa é `orgao_de_pesquisa_sem_opcao_selecionada_na_tela`.
Regra do HTML, não observada nas cassettes; teste:
`opcao_de_orgao_selecionada_mas_desabilitada_nao_e_publicada`.

As checkboxes marcadas, os selects simples e os textos vazios que o
navegador enviaria continuam fora do POST. Na UnB 4.0.9 o servidor aceitou
o POST assim mesmo (03/11/2026); nas outras instalações, medir isso exige a
[coleta do formato de POST](../coleta-formato-post.md) ou ablação em
treinamento autorizado.

## Linhas ausentes da faixa

Decisão do proprietário (04/11/2026): compatibilidade com o máximo de
instalações e versões. Goiás 5.0.3 (`processo-busca-go#4`) responde
"Exibindo 1 - 10 de 966" com 9 registros. O HTML gravado tem 27 `tr`
(três por registro), nenhum link de processo fora dos registros lidos e
nenhuma forma de linha diferente: a linha não está na resposta, e o
leitor não a pula.

- **Alternativas:** (a) recusar como antes
  (`linhas_da_tabela_diferentes_da_faixa_da_barra`); (b) publicar as
  linhas presentes como se fossem a faixa; (c) publicar as linhas
  presentes, a faixa da barra e quantas posições faltam.
- **Escolha: (c).** A lista sai com `faixa_observada` = a faixa
  **escrita** na barra, `total_declarado` da barra e
  `linhas_ausentes_da_faixa` = `F - I + 1 -` linhas lidas (0 quando
  completa). Nunca se diz que a página está completa; `--pagina N`
  continua contando pela barra (tamanho e `hdnInicio`), e a página N
  confere a faixa da barra contra a esperada.
- **Fora da regra:** mais linhas que a faixa e qualquer diferença quando a
  barra é "N resultados" (a faixa é só implícita) continuam
  `linhas_da_tabela_diferentes_da_faixa_da_barra`. As conferências de
  link (`link_de_processo_fora_da_linha_de_pesquisa`,
  `link_de_processo_sem_classe_reconhecida`) seguem impedindo que uma
  linha com link seja perdida em silêncio.
- **Trade-off:** uma linha com forma nova **sem** nenhum link de
  processo seria contada como ausente em vez de recusada; o campo a
  expõe, e a regra vale também para `ies documento buscar`.
- **Revisão:** se uma instalação mostrar por que a linha falta (aviso,
  marcador de sigilo), trocar a contagem pelo fato.
- **RED:** 6 testes falhando (resposta com 9 de 10, tabela vazia com
  barra, documentos, página 2 de documentos e o texto humano das duas
  CLIs) antes da implementação; o replay do acervo passou de 5 recusas
  de Goiás a 0 ([processo](../validacao-replay/2026-11-04-processo-buscar-acervo.md),
  [documento](../validacao-replay/2026-11-04-documento-buscar-acervo.md)).

## Limitações e revisão

- Ensaio live **do comando IES** (03/11/2026,
  [relatório](../validacao-live/2026-11-03-processo-buscar.md)): PASS na
  UnB e em GO nos cinco casos; a PEN1 responde `sem_resultado` até para
  NUP da própria unidade (resposta do servidor, reproduzida fora do IES).
  PEN3 5.0.4 (04/11/2026,
  [relatório](../validacao-live/2026-11-04-processo-buscar.md)): PASS,
  inclusive NUP fora da caixa da unidade. UFRJ, PEN2 e PEN4: NOT_RUN.
- Linhas a menos que a faixa da barra são publicadas com
  `linhas_ausentes_da_faixa` (ver abaixo); a causa (filtro de acesso
  depois da contagem do índice, presumido) não está na tela.
- A página N e a barra acima de 999 (`3.441`, `15.835`) foram vistas ao
  vivo; uma barra sem separador continua falhando fechada.
- Não há filtros além do protocolo. O termo obedece ao `maxlength`
  **publicado** em `txtProtocoloPesquisa` (50 nas capturas da UnB e da
  PEN1), contado em unidades UTF-16 como no navegador; acima dele a recusa
  vem antes do POST. O valor não é presumido: sem atributo, a tela não
  limita; atributo fora de dígitos ASCII é recusa (mais estrito que o
  navegador, que o ignoraria). A
  [ablação](../como-o-sei-funciona/pesquisa-de-protocolo.md) mostrou que o
  servidor aceita e usa o termo acima de 50: o limite é do cliente, e a
  regra fica por fidelidade à tela e para não gastar 3 requisições num
  termo que nenhum NUP precisa.
- Aviso de "sem resultado" junto com barra de contagem é resposta
  ambígua e falha fechada, como tabela com aviso.
- Revisar se outra instalação publicar o menu em outra forma, a barra em
  outro texto, ou se 1 resultado por texto vier no singular.
- Não reutilizar esta decisão como prova de autenticação, de inexistência
  do processo ou de resultado integral.
