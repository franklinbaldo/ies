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
  `infra_unidade_atual`), mesmo id; a sessão sobrevive.
- Recusadas com 302 para `login.php` e `msg=Link sem assinatura`, **e a
  sessão morreu** (controle depois falhou), cada uma:
  `procedimento_controlar` sem parâmetros, `protocolo_pesquisar` sem
  parâmetros, `procedimento_trabalhar&id_documento=M` (sem
  `id_procedimento`), `documento_visualizar&id_documento=M`,
  `procedimento_consultar_historico&id_procedimento=N` e
  `arvore_visualizar&id_procedimento=N`.

Consequência: a **única** entrada sem assinatura é `procedimento_trabalhar`
por id. O Todos também mediu como aceita a mesma forma acrescida de
`&id_documento=M`; o IES não a mediu e não a monta. Todo o resto vem da
tabela `link_assinado` da sessão. O Todos mediu ainda que os links do
menu deixam de valer depois de trocar de unidade (o hash cobre
`infra_unidade_atual`): a unidade é parte da chave do link.
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

`origem` é a identidade publicada da instalação (`https://host[/base]`,
de `Instalacao::url_base`). O NUP é texto opaco, comparado byte a byte,
sem normalização. O `id_procedimento` também é opaco, mas só se guarda
quando é dígito ASCII (1 a 20): ele vai para uma URL montada pelo
chamador e não pode carregar `&` ou `=`.

### Regras de operação

- **O servidor vence.** O índice é um palpite barato, validado a cada
  uso pelo chamador (a árvore aberta tem de mostrar o NUP pedido; o link
  guardado tem de responder com a tela esperada). Divergência apaga a
  linha (`esquecer_processo`, `esquecer_link`) e o comando segue pelo
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
  variante de `RegistroRecusado`, sem dados. Nenhuma expressão regular.
- **`geracao_sessao`** é uma chave opaca do chamador (será o SHA-256 do
  cookie, como `cookie_sha256` do marcador de geração do login). O
  índice não lê arquivos de sessão. `link()` só devolve linhas da
  geração pedida; `descartar_outras_geracoes` apaga as demais da
  instalação.

### Entrada por id: risco e política

Uma recusa derruba a sessão **para todos os agentes da conta**, não só
para o comando que tentou. Por isso:

- o módulo torna impossível montar outra URL sem assinatura: uma função
  só, `url_da_entrada_por_id(instalacao, id_procedimento)`, sem outros
  parâmetros, com o id restrito a dígitos ASCII; um teste confere a forma
  exata e outro que nenhum outro arquivo do módulo contém
  `controlador.php?acao=`;
- a entrada por id só é usada onde `entrada_por_id` devolve `Some(true)`;
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

## Privacidade

O arquivo revela quais processos a pessoa acessa e guarda URLs que
valem enquanto a sessão viver.

- Caminho: `IES_INDICE_FILE` se definido e não vazio; senão
  `ies.sqlite3` no diretório do `IES_URL_CONTROLE_FILE`
  (`ArquivosDeAcesso::url_controle`). `IES_SEM_INDICE` com valor não
  vazio diferente de `0` desliga.
- O diretório é criado `0700`; diretório existente com bits de grupo ou
  outros desliga o índice (o IES não muda o modo de diretórios que não
  criou). O banco nasce `0600` com `O_CREAT|O_EXCL` (não segue link simbólico); banco, `-wal` ou
  `-shm` que seja link simbólico, não seja arquivo regular ou tenha
  bits de grupo/outros desliga o índice. O SQLite abre com
  `SQLITE_OPEN_NOFOLLOW` e sem `CREATE`; `-wal` e `-shm` herdam o modo.
- URL assinada só sai por `UrlAssinada::url()`; `Debug` de
  `UrlAssinada`, `RegistroRecusado` e `IndiceLocal` não mostra URL,
  hash, geração nem cookie. O índice não escreve logs nem devolve o
  erro do SQLite.
- Apagar o arquivo é sempre seguro: tudo volta na próxima leitura.

## Modelo de falha

Arquivo corrompido, permissão, disco cheio, versão futura ou lock
além do `busy_timeout` **nunca** derrubam um comando: a primeira falha
desliga o índice daquele processo, e as consultas passam a devolver
`None` e as gravações a não fazer nada. A API não propaga erro do banco;
`RegistroRecusado` só descreve entrada inválida, e o chamador pode
ignorá-la.

- Cada operação abre a conexão, roda **uma transação minúscula** e
  fecha. Nenhuma transação atravessa HTTP: o chamador consulta, faz a
  requisição e só depois registra.
- `journal_mode=WAL` e `busy_timeout` de 2 s permitem vários `ies` ao
  mesmo tempo no mesmo banco.

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

- A integração (pesquisa, `processo`, login) vem em PRs seguintes; este
  só entrega o módulo, testado offline com diretórios temporários.
- O módulo não depende de `pesquisa_web`, `processo*` nem `sessao_login`.

## Condições de revisão

- Instalação de treinamento em que a entrada por id aceita e depois
  recusa (a regra de duas seguidas pode ser pouca ou demais).
- Medição de hash de uso único ou dependente de navegação: links
  assinados deixariam de ser guardados.
- Validação live: **NOT_RUN** nesta fatia (sem integração).
