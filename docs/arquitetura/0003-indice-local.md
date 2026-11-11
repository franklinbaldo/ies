---
type: ArchitectureDecision
title: Índice local SQLite de identidade, capacidade e links assinados
description: Palpite barato validado a cada uso, com três tabelas de vidas distintas e falha que nunca derruba um comando
status: proposed
decision: sqlite-index three-lifetimes server-wins fail-silent
---

# 0003 — Índice local (`src/indice_local.rs`)
## Problema

O princípio de **mínimo de requests** das [instruções](../../AGENTS.md)
esbarra em três custos repetidos a cada comando:

1. **Achar o processo pelo NUP.** Sem saber o `id_procedimento`, o IES
   precisa da pesquisa assinada (Controle → link da pesquisa → POST →
   302 → árvore). O NUP é a chave que a pessoa digita; o id é o que a
   entrada direta aceita.
2. **Saber se a instalação aceita a entrada por id.** Ver a medição do
   IES abaixo; o Todos mediu o mesmo aceite em PEN3, PEN4 e GO, e as
   recusas de `acao_origem` sem hash, `procedimento_visualizar` e
   `protocolo_pesquisa=NUP`
   ([Todos, infra-hash](https://github.com/franklinbaldo/todos/blob/main/docs/como-o-sei-funciona/03-infra-hash.md),
   itens 4 e 5).

**Evidência própria do IES** (UnB 4.0.9, treinamento, 03/11/2026, só GETs
de leitura, sessão nova por caso, controle positivo antes e depois):

- `controlador.php?acao=procedimento_trabalhar&id_procedimento=N` sem
  hash → **302 para a mesma ação assinada pelo servidor** (chaves `acao`,
  `id_procedimento`, `infra_hash`, `infra_sistema`,
  `infra_unidade_atual`), mesmo id; a sessão sobrevive. Idem com
  `&id_documento=M` acrescido (`procedimento_trabalhar` com os dois ids).
- Recusadas com 302 para `login.php` e `msg=Link sem assinatura`, **e a
  sessão morreu** (controle depois falhou), cada uma na forma mínima
  (`acao` + ids, sem contexto): `procedimento_controlar` e
  `protocolo_pesquisar` sem parâmetros,
  `procedimento_trabalhar&id_documento=M` (sem `id_procedimento`),
  `documento_visualizar`, `documento_imprimir_web`,
  `documento_versao_listar`, `assinatura_listar`, `arvore_visualizar`
  (com e sem documento), `procedimento_consultar_historico`,
  `comentario_listar`, `acompanhamento_gerenciar`,
  `andamento_marcador_gerenciar`, `protocolo_modelo_gerenciar`,
  `rel_bloco_protocolo_listar&id_bloco=B` (conteúdo do bloco) e
  `bloco_interno_listar`. Candidatas escolhidas do inventário de 112
  `acao` publicadas nas capturas, só as de leitura; nenhuma de escrita.

Consequência: a **única** entrada sem assinatura é `procedimento_trabalhar`
por id do processo, opcionalmente com o do documento (17 formas medidas,
2 aceitas). O módulo monta só essas duas formas: a com `id_procedimento`
(`processo NUP`) e, desde a fatia 2 do #60, a com `&id_documento=M`
(`documento DOC` pelo índice), cada uma com o aceite registrado à parte
na tabela `instalacao` (a com documento só foi medida na UnB). Todo o resto vem da
tabela `link_assinado` da sessão. O Todos mediu ainda que os links do
menu deixam de valer depois de trocar de unidade (o hash cobre
`infra_unidade_atual`): a unidade é parte da chave do link.
Com o `ies processo NUP` integrado (03/11/2026), a entrada por id foi
aceita também em GO 5.0.3 e na PEN1 5.0.4, que o Todos não tinha medido
([relatório](../validacao-live/2026-11-03-processo-obter-arvore.md)).
3. **Reabrir telas já vistas na sessão.** Cada tela publica links
   assinados para as próximas; sem guardá-los, reabrir um processo
   recomeça do Controle.

Evidência de comportamento (oráculo, não arquitetura a portar):
[`indice.py`](https://github.com/franklinbaldo/todos/blob/main/src/todos/sei/indice.py)
(docstring, esquema, `entrada_por_id`, `registrar_entrada_por_id`),
[13-indice-local](https://github.com/franklinbaldo/todos/blob/main/docs/como-o-sei-funciona/13-indice-local.md),
[11-otimizacoes](https://github.com/franklinbaldo/todos/blob/main/docs/como-o-sei-funciona/11-otimizacoes.md)
e [03-infra-hash](https://github.com/franklinbaldo/todos/blob/main/docs/como-o-sei-funciona/03-infra-hash.md).
O item 3 deste último registra que, **dentro de uma sessão, o hash não é
de uso único e é determinístico**; em outra sessão o mesmo link tem
outro hash. Medido hoje pelo agente pai na UnB: a URL da árvore colhida
antes, reaberta depois de navegar por outras telas, respondeu **200**;
a mesma URL depois de um novo login respondeu **302**.

## Decisão

Um banco SQLite local (`rusqlite` com `bundled`, sem as features
padrão) com três tabelas de vidas diferentes. Esquema versionado por
`PRAGMA user_version` (versão 1), migração só para frente; banco de
versão mais nova que o binário desliga o índice sem tocar nele.

| Tabela | Chave | Vida | Uso |
|---|---|---|---|
| `instalacao` | `origem` | durável; recusa expira em 30 dias | a instalação aceita a entrada por id? |
| `processo` | `(origem, nup)`, `(origem, id_procedimento)` únicos | durável até divergir | NUP ↔ id |
| `link_assinado` | `(origem, geracao_sessao, unidade, acao, objeto)` | só a geração de sessão em que foi visto | URL assinada **literal** |
| `documento` (versão 2) | `(origem, numero_sei)`, `(origem, id_documento)` únicos | durável até divergir | número SEI ↔ (`id_procedimento`, `id_documento`) |

`origem` é a identidade publicada da instalação (`https://host[/base]`,
de `Instalacao::url_base`). O NUP é texto opaco, comparado byte a byte,
sem normalização. O `id_procedimento` também é opaco, mas só se guarda
quando é dígito ASCII (1 a 20): ele vai para uma URL montada pelo
chamador e não pode carregar `&` ou `=`. O mesmo vale para o
`id_documento`. O número SEI é opaco (zeros à esquerda contam) e nunca
vai para URL. A tabela `documento` aprende de toda árvore conferida
(`processo NUP` e `documento DOC`), numa transação; número ou id repetido
na mesma árvore é pulado. A migração 1 → 2 cria a tabela e as colunas
`entrada_com_documento`, `entrada_com_documento_testado_em` e
`recusas_com_documento` de `instalacao`.

### Regras de operação

- **O servidor vence.** O índice é um palpite barato, validado a cada
  uso pelo chamador (a árvore aberta tem de mostrar o NUP pedido; o link
  guardado tem de responder com a tela esperada). Divergência apaga a
  linha (`esquecer_processo`, `esquecer_link`, `esquecer_documento`) e o comando segue pelo
  caminho sem índice.
- **Nunca uma requisição só para alimentá-lo.** Ele aprende com
  respostas que o comando já leu.
- **Nova observação conflitante substitui.** Registrar `(nup, id)` apaga
  na mesma transação a linha do mesmo NUP e a do mesmo id.
- **Links nunca são montados nem editados.** O chamador entrega a URL
  absoluta que a tela publicou; o índice a confere e guarda como veio.
  Na gravação: `https://`, host e caminho de `controlador.php` aceitos
  por `Instalacao::autoriza_controlador`, bytes 33–126 sem `#` nem `\`,
  chaves canônicas e valores sem `;` (as mesmas funções do
  `transporte`), exatamente um `acao` igual à coluna, exatamente um
  `infra_hash` de 64 hexadecimais, nenhum `acao_ajax`, e
  `infra_unidade_atual`, se presente, igual à unidade. Cada recusa é uma
  erro de `RegistroRecusado`, sem dados, que o índice guarda nos avisos
  da saída (#786). Nenhuma expressão regular.
- **`geracao_sessao`** é uma chave opaca do chamador (será o SHA-256 do
  cookie, como `cookie_sha256` do marcador de geração do login). O
  índice não lê arquivos de sessão. `link()` só devolve linhas da
  geração pedida; `descartar_outras_geracoes` apaga as demais da
  instalação.

### Entrada por id: risco e política

Uma recusa derruba a sessão **para todos os agentes da conta**, não só
para o comando que tentou. Por isso:

- o módulo torna impossível montar outra URL sem assinatura:
  `url_da_entrada_por_id(instalacao, id_procedimento)` e
  `url_da_entrada_por_id_com_documento(instalacao, id_procedimento,
  id_documento)`, que só acrescenta `&id_documento=M` à primeira, com os
  ids restritos a dígitos ASCII; testes conferem as formas exatas e que
  nenhum outro arquivo do módulo contém `controlador.php?acao=`;
- a forma com documento só é tentada onde nenhuma das duas foi recusada;
- a entrada por id só é usada onde `entrada_por_id` devolve
  `Some(AceiteDaEntrada::Aceita)`;
- instalação desconhecida (`None`) aprende **uma vez**: o primeiro
  comando que precisaria da pesquisa tenta a entrada por id e registra
  o resultado;
- recusa característica ("Link sem assinatura" na tela de login) vira
  erro tipado que manda rodar `ies login`; **nunca relogin automático**;
- a recusa expira em 30 dias (um upgrade do SEI muda o comportamento) e
  a instalação volta a `None`.

**Regra de contagem de recusas** (diferente do Todos, que exige duas
seguidas em qualquer caso):

- instalação **desconhecida ou com recusa expirada**: **uma** recusa
  característica basta para marcá-la como recusada. Cada tentativa custa
  a sessão de todos; exigir duas faria a instalação que recusa derrubar
  duas sessões antes de o índice aprender;
- instalação **já marcada como aceita**: só **duas recusas seguidas**
  a rebaixam. Uma recusa isolada numa instalação que aceitou pode ser
  sessão vencida respondida de outro jeito (a razão do Todos); um aceite
  zera a conta.

O chamador só registra recusa quando a assinatura é a característica;
302 para o login sem essa mensagem é sessão vencida, não recusa.

### Por que links assinados aqui, se o Todos os exclui

O Todos não persiste URL assinada por tratá-la como autoridade de
sessão. O IES guarda com escopo de sessão porque:

- a vida é a da sessão: a chave inclui a geração, e um login novo
  descarta as outras gerações. Fora da sessão o hash não vale (302);
- a sensibilidade é a mesma do arquivo de cookie, que já fica em disco
  `0600`: quem lê o banco poderia ler o cookie ao lado;
- o ganho é medido: `processo NUP` cai de **5 para 1 request** dentro de
  uma sessão quando a URL da árvore já foi vista;
- o hash dentro da sessão é reutilizável e não é de uso único
  (03-infra-hash, item 3; medição de hoje na UnB, acima).

### Colheita do Controle (zero requisições)

Todo comando que **já** lê o Controle de Processos
(`procedimento_controlar`) entrega cada página que leu a
`colheita_do_controle::colher` (`src/colheita_do_controle.rs`), que
alimenta o índice com o que a página mostra. Nenhuma requisição a mais,
nenhuma página a mais (`--pagina N` colhe só as páginas de fato lidas) e
nenhuma mudança na saída ou no código de saída do comando.

| O que | Tabela | Chave | Vida |
|---|---|---|---|
| NUP ↔ id de cada linha válida | `processo` | `(origem, nup)` | durável até divergir |
| `href` da linha (`procedimento_trabalhar`) | `link_assinado` | objeto = id | geração e unidade |
| menu `protocolo_pesquisar` | `link_assinado` | objeto vazio | geração e unidade |
| `action` de `frmProtocoloPesquisaRapida` | `link_assinado` (`protocolo_pesquisa_rapida`) | objeto vazio | geração e unidade |

- **Leitores reaproveitados.** As linhas vêm dos mesmos leitores da caixa:
  visão R (`tblProcessosRecebidos` e `tblProcessosGerados`, `caixa.rs`) e
  visão D (`tblProcessosDetalhado`, `caixa_detalhada.rs`), que agora
  devolvem também o `href` do link do número. Página que o leitor recusa
  não alimenta nada.
- **Conferência por linha.** O `href` decodificado tem de ser relativo
  (`controlador.php?`) e passar por `conferir_trabalho` na forma
  `OrigemDoTrabalho::Controle` (uma `acao=procedimento_trabalhar`,
  `acao_origem=procedimento_controlar`, `id_procedimento` igual ao id da
  linha, `infra_hash` de 64 hex, `infra_unidade_atual` e `infra_sistema`
  iguais aos da URL privada do Controle, host da instalação) e depois pela
  conferência de gravação do índice. Linha que falha é pulada (nem o par
  NUP ↔ id é gravado); NUP repetido na página, ou id em duas linhas, pula
  todas as linhas envolvidas. Nada disso falha o comando.
- **Chave.** A geração é o SHA-256 do cookie em uso (`geracao_da_sessao`);
  a unidade, o `infra_unidade_atual` da URL privada do Controle — as
  mesmas de `ies processo NUP`. No `ies login`, a colheita roda depois
  de gravar a sessão, com o cookie e a URL recém-gravados.
- **Onde.** `ies login` (login novo e sessão gravada válida),
  `ies status --verificar-controle-web`, `ies unidade atual`,
  `ies unidade listar-processos`, `recebidos`, `gerados`,
  `ies unidade colunas [--definir]` (GET inicial e telas do Controle
  devolvidas pelos POSTs de visão e página), o GET do Controle do
  passo 4 de `ies processo NUP` e o de `ies processo buscar`.
- **Uso.** O link `procedimento_trabalhar` colhido abre o passo 1b de
  `ies processo NUP` (2 GET até a árvore,
  [contrato](../../contracts/cli/processo-obter-arvore.md)). O link da
  pesquisa avançada fica guardado; nenhum comando o usa ainda.

## Alternativas

1. **Sem índice.** Zero estado, mas cada `processo NUP` custa a
   pesquisa inteira e a entrada por id nunca é aprendida.
2. **Arquivos JSON por tabela.** Sem dependência, mas sem atomicidade
   entre processos concorrentes; reimplementaria lock e gravação atômica.
3. **Só identidade, como o Todos.** Perde o ganho de 5 → 1 dentro da
   sessão.
4. **SQLite com três vidas** (escolhida). Custo: compilar o SQLite em C
   (`cc`). As features padrão do `rusqlite` (`cache`, `hashlink`) ficam
   fora: o módulo não usa statements em cache. (Elas tinham saído para o
   `Cargo.lock` caber em 20.000 bytes; depois os `.lock` ficaram isentos
   do limite, [CI](../ci.md).)

## Consequências

- A integração vem por comando: `ies processo NUP` (Refs #59, fatia 1)
  usa as três tabelas
  ([contrato](../../contracts/cli/processo-obter-arvore.md)); os comandos
  que leem o Controle só o alimentam (colheita, acima).
- O módulo não depende de `pesquisa_web`, `processo*` nem `sessao_login`;
  a colheita, que depende dos leitores, fica fora dele.

## Condições de revisão

- Instalação de treinamento em que a entrada por id aceita e depois
  recusa (a regra de duas seguidas pode ser pouca ou demais).
- Medição de hash de uso único ou dependente de navegação: links
  assinados deixariam de ser guardados.
- Validação live: integração validada nos ensaios de `processo NUP` e da colheita do Controle ([relatório](../validacao-live/2026-11-03-colheita-do-controle.md)).

O esquema 3 e o contrato SQL público de leitura das unidades estão em
[unidades no índice local](0003-indice-local-unidades.md).

## Partes da decisão

- [Índice local — privacidade e modelo de falha](0003-indice-local/privacidade-e-falha.md): privacidade, modelo de falha e auditoria complementar #188 do índice local.
