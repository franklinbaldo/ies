---
type: ImplementationDecision
title: Leitura coerente de URL e cookie da sessão, com o marcador antes e depois
operation: sessao.login
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de sessao.login (Refs #222); toda leitura dos arquivos de sessão pelos comandos que não são o ies login (ReferenciasDaSessao) lê o marcador .geracao antes e depois de URL, origem e cookie, relê os arquivos locais uma vez se o marcador mudou ou não confere com o par, e falha com sessao_trocada_durante_a_leitura se mudou de novo; o cookie lido vai com a URL até a última requisição do comando e até a colheita do índice; sessão sem marcador mantém o comportamento anterior
selected_strategy: marcador-antes-e-depois-com-uma-releitura-local
alternatives: marcador-antes-e-depois-com-uma-releitura-local,lock-compartilhado-na-leitura,cookie-relido-a-cada-requisicao,releitura-ilimitada
evidence_level: testes-sinteticos-com-intercalacao-deterministica-dos-escritores-de-producao
live_validation: not-run-so-arquivos-locais
---

# Leitura coerente da sessão

Achado da auditoria #183, aberto como
[issue #222](https://github.com/franklinbaldo/ies/issues/222). Contrato:
[gravação, geração e lock](../../../contracts/cli/sessao-login.md#gravação-geração-e-lock).

## Problema

`ReferenciasDaSessao::ler` lia a URL do Controle e a origem; o cookie
era aberto depois, por cada requisição (`cookie_privado_validado`). O
`ies login` (e o relogin da #200) e o `ies unidade trocar --confirmar`
(#205) gravam sob o lock do login, mas os leitores não o pegam. Entre as
duas leituras, um deles podia trocar os arquivos: o comando mandava a URL
da geração A com o cookie da B, e a colheita do índice relia os arquivos
por conta própria e podia registrar links sob a geração errada.

## Alternativas consideradas

- **marcador-antes-e-depois-com-uma-releitura-local (escolhida):** lê o
  marcador, a URL, a origem (conferida antes de abrir o cookie), o cookie
  e o marcador de novo. Coerente quando os dois marcadores são iguais e,
  havendo marcador, os SHA-256 dele conferem com a URL e o cookie lidos.
  O marcador é renomeado por último pelo escritor; um marcador estável que
  não confere é uma gravação no meio (ou interrompida). Incoerente: relê
  os arquivos locais uma vez (disco, nenhuma requisição). Incoerente de
  novo: `sessao_trocada_durante_a_leitura`, sem rede e sem relogin.
- **lock-compartilhado-na-leitura:** os leitores pegariam o lock do login
  em modo compartilhado. O lock é um `flock` exclusivo de
  `<url-controle>.lock`, criado por quem o pede: o leitor passaria a
  criar arquivo e a esperar até 120 s por um login alheio, e um leitor
  lento bloquearia o login. Rejeitada.
- **cookie-relido-a-cada-requisicao (situação anterior):** cada requisição
  relia o cookie; um comando de várias requisições podia mudar de geração
  no meio. Rejeitada: o cookie lido no retrato vai até a última
  requisição.
- **releitura-ilimitada:** reler até ficar coerente. Um escritor em laço
  prenderia o leitor; uma gravação interrompida, para sempre. Rejeitada.

## O que muda

- `ReferenciasDaSessao::ler(url, origem, cookie)` devolve URL, origem e
  cookie do mesmo retrato. O cookie é um `CookieDaSessao`, redigido no
  `Debug`. Nenhuma requisição dos comandos lê o arquivo do cookie: todas
  recebem o do retrato.
- `Colheita::das_referencias` usa o mesmo retrato: o link colhido vai para
  a geração do cookie que obteve a página.
- `unidade trocar --confirmar` regrava a URL com o cookie do retrato.
- Falhas de origem e de cookie (ausente, permissão, sintaxe) passam a
  sair na leitura, antes da primeira requisição, com os mesmos
  subcódigos.
- O `ies login` não usa o retrato: lê sob o lock que os escritores pegam.

## Lacuna: sessão sem marcador

Sessão preparada fora do `ies login` (três arquivos escritos à mão, sem
`<url-controle>.geracao`) não tem geração para comparar: a leitura é a de
antes, sem releitura, e a mistura continua possível se alguém regravar à
mão no meio. O primeiro `ies login` grava o marcador e fecha a lacuna.

## Verificação

Testes em `src/sessao_login/tests_snapshot.rs`, com os escritores de
produção (`gravacao::gravar` e `regravar_url_do_controle`) chamados entre
a URL e o cookie por um gancho determinístico: relogin uma vez (relê,
geração B inteira), troca de unidade uma vez (relê, URL nova), troca nas
duas leituras (falha tipada, `sessao_encerrada` não dispara relogin),
gravação interrompida (falha tipada), sem marcador (lacuna) e origem
divergente (cookie nunca aberto). A colheita grava sob a geração A mesmo
com B gravada antes de colher. Em `src/processo_obter/tests_sessao.rs`,
um relogin concorrente no meio de `processo NUP` não muda o cookie das
requisições seguintes. Nada novo vai ao SEI; não há ensaio live (só
arquivos locais).

## Revisitar quando

- a gravação deixar de renomear o marcador por último;
- surgir leitor de sessão que não passe por `ReferenciasDaSessao`.
