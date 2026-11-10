---
type: ImplementationDecision
title: Relogin único depois de sessão vencida, só nos comandos de leitura
operation: sessao.login
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de sessao.login (Refs #200); as nove leituras (processo NUP, processo buscar, documento DOC, documento buscar, unidade listar-processos, recebidos, gerados, atual e colunas sem --definir) relogam no máximo uma vez, só depois de pagina_de_login_recebida_no_controle ou sessao_encerrada_rode_ies_login (também o 302 do GET do Controle para a entrada do login da instalação, revisão de 05/11/2026), com as credenciais do arquivo 0600 do ies login, conferindo a geração gravada sob o lock antes de logar, e recomeçam do início uma vez; escrita e prévia de escrita nunca relogam
selected_strategy: relogin-unico-com-geracao-sob-lock
alternatives: relogin-unico-com-geracao-sob-lock,sem-relogin-agente-roda-ies-login,relogin-sem-conferir-geracao,relogin-tambem-na-escrita,relogin-por-qualquer-falha-de-sessao
evidence_level: testes-sinteticos-com-sequencia-exata-e-ensaio-live-pen4
live_validation: pass-pen4-sessao-desconhecida-pelo-servidor-geracao-trocada-e-302-do-controle-vencimento-natural-not-run
---

# Relogin único nas leituras

Desenho aprovado pelo proprietário na
[issue #200](https://github.com/franklinbaldo/ies/issues/200) em
04/11/2026; esta decisão registra como ele foi implementado e por quê.
Contrato: [relogin único nas leituras](../../../contracts/cli/sessao-login.md#relogin-único-nas-leituras).

## Alternativas consideradas

- **sem-relogin-agente-roda-ies-login (situação anterior):** a leitura sai
  com `sessao_encerrada_rode_ies_login` (ou
  `pagina_de_login_recebida_no_controle`) e o agente roda `ies login`.
  Zero código, mas com vários agentes na mesma conta cada um loga por
  conta própria, e cada login são 5 a 7 pedidos.
- **relogin-unico-com-geracao-sob-lock (escolhida):** depois da falha
  tipada, o lock do `ies login`; sob ele, a geração gravada é comparada
  com a do início do comando. Mudou: outro processo já relogou, a leitura
  usa a sessão nova sem login. Igual: um login, gravação da nova geração e
  a leitura recomeça do início, uma vez.
- **relogin-sem-conferir-geracao:** relogar sempre que a leitura falha.
  Dois agentes que veem a mesma sessão vencida fariam dois logins
  seguidos; o segundo descartaria a sessão que o primeiro acabou de gravar.
- **relogin-tambem-na-escrita:** relogar no `--confirmar` ou na prévia.
  Misturaria sessões entre a prévia e a escrita, e uma escrita não confirmada não
  pode ser repetido (AGENTS.md). Rejeitada no desenho.
- **relogin-por-qualquer-falha-de-sessao:** relogar também em 401/403,
  3xx, WAF ou tela inesperada. O 403 da borda (GO, Cloudflare) não é
  sessão vencida: relogar aumentaria a rajada que causou o bloqueio.

## Comparação

| Critério | sem relogin | **geração sob lock** | sem geração | também escrita | qualquer falha |
| --- | --- | --- | --- | --- | --- |
| Logins com N agentes e a mesma sessão vencida | N | **1** | N | 1 | N |
| Pedidos a mais por relogin | 0 (mas o `ies login` à parte) | login + releitura | idem | idem | idem, também no 403 |
| Risco na escrita | nenhum | **nenhum** | nenhum | mistura de sessões | mistura de sessões |
| Rajada sob WAF/Cloudflare | não | **não** | não | não | sim |
| Código novo | 0 | um orquestrador e a borda | menor | maior | igual |

A comparação de geração reaproveita o que já existia: o lock exclusivo
do `ies login` (`<url-controle>.lock`) e a gravação atômica com o
marcador por último. A "geração" comparada são os SHA-256 do cookie e da
URL do Controle lidos dos arquivos (os mesmos campos do marcador), e não
o contador do marcador: assim vale também para sessão preparada à mão,
sem marcador.

## Decisão

1. Um orquestrador sem rede própria
   ([`src/sessao_login/relogin.rs`](../../../src/sessao_login/relogin.rs)):
   lê a geração gravada, roda a leitura, e só se o subcódigo é um dos
   dois de sessão encerrada **e** o arquivo de credenciais existe, chama
   o relogin de [`fluxo.rs`](../../../src/sessao_login/fluxo.rs) (lock,
   conferência da geração, login sem o GET de conferência, `--unidade`
   conferida antes de gravar, gravação e colheita). A leitura roda de novo
   uma vez; o resultado, qualquer que seja, é o do comando.
2. A classificação é pelo **subcódigo tipado**, não pela mensagem: cada um
   dos dois nasce de uma única variante (`PaginaDeLoginRecebidaNoControle`
   do transporte e `SessaoEncerrada` dos fluxos), inclusive aninhada nos
   erros de cada leitura. Ficam fora `entrada_por_id_recusada_rode_ies_login`
   (a recusa característica derruba a sessão, não é vencimento),
   `tela_de_login_recebida_apos_envio` (resposta de POST) e
   `redirecionamento_recusado`.
3. A borda ([`src/relogin_cli.rs`](../../../src/relogin_cli.rs)) dá o
   transporte real do login, `IES_CREDENCIAIS_FILE`, a espera de 120 s do
   lock e a contagem de **todos** os pedidos do processo: um contador no
   cliente HTTPS único, registrado antes do envio. `requisicoes` soma a
   primeira tentativa, o login e a releitura; `relogin: true` só quando
   este processo logou. Cada leitura relê sessão, índice e colheita a cada
   tentativa (a colheita é chaveada pela geração).
4. Os cinco comandos de escrita não chamam a borda; um teste confere a
   ligação nos oito módulos de leitura e a ausência nos cinco de escrita.

## Evidências verificáveis

- Testes da sequência exata, com login e leitura num só roteiro
  ([`tests_relogin.rs`](../../../src/sessao_login/tests_relogin.rs)):
  vencida → `GET` da leitura, os 5 pedidos do login da família UnB,
  `GET` com a sessão nova (7; `antes` 6, `releitura` 1); geração trocada
  por outro processo → `GET`, `GET` com a sessão nova, sem login e sem
  marcador gravado; segunda falha → os mesmos 7 e o erro, roteiro
  esgotado; sem credenciais (variável ausente ou arquivo ausente) → 1
  pedido, sem lock; 403, WAF, rede, 302 fora do login (outro host, outro caminho,
  fragmento, sem `Location`), 301, 429 e tela inesperada → 1 pedido,
  sem lock; credencial recusada e unidade diferente → erro do login, sem
  releitura e sem gravar.
- Saída e ligação ([`relogin_cli/tests.rs`](../../../src/relogin_cli/tests.rs)):
  `relogin`, `requisicoes`, a lista D e a falha depois do relogin; as
  escritas sem relogin.
- RED: 11 dos 14 testes novos falharam com o shim compilável (os 3 que
  passavam afirmam o comportamento anterior: sem credenciais, outras
  falhas, saída sem recomeço); GREEN: 1.275 testes da `ies`.
- Teste de contrato existente do binário: [`cli_contract.rs`](../../../tests/cli_contract.rs).
- [Ensaio live na PEN4](../../validacao-live/2026-11-05-relogin.md).
- [Evidência do login](../../../contracts/evidencias/sessao-login.md).

## Limitações e revisão

- **Vencimento natural não observado.** Na PEN4, uma sessão aberta 9 h
  antes (e ociosa por mais de 2 h) e a sessão anterior a um
  `login --forcar` continuavam válidas. O relogin live foi disparado por
  uma sessão que o servidor não conhece (o `PHPSESSID` de uma cópia
  trocado por outro valor): a entrada por id respondeu 302 para a tela de
  login, `sessao_encerrada_rode_ies_login`, e o relogin aconteceu.
- **Achado (resolvido na revisão abaixo):** na PEN4, o GET do Controle
  com essa sessão respondeu **302** (`redirecionamento_recusado`), e as
  leituras que começam pelo Controle não relogavam.
- **Várias contas** na mesma instalação não mudam nada: o lock e a
  geração são por arquivo de sessão. **Sessão única no SIP**: sem
  evidência (na PEN4, as duas sessões valiam juntas).
- **Cloudflare (PEN2 a PEN4):** o relogin acrescenta o login (6 pedidos
  na família PEN) à rajada do comando; nenhum bloqueio no ensaio.
- A comparação de geração tem uma janela: se outro processo grava entre a
  leitura da geração e a primeira leitura da sessão, a releitura usa a
  sessão que acabou de falhar e sai com o erro. Rever se aparecer.
- Rever se o SEI publicar outra forma de sessão vencida (outra entrada
  de login no `Location` do Controle, por exemplo), se a escrita
  ganhar fluxo idempotente verificável ou se o lock precisar de outro
  arquivo.

## Revisão: 302 do Controle para o login

Aprovada pelo proprietário em 05/11/2026 (Refs #200): corrigir a
**classificação**, não alargar o gatilho.

**Alternativas.**

- **302-para-o-login-com-o-mesmo-subcodigo (escolhida):** o 302 do GET do
  Controle cujo `Location`, resolvido contra a instalação, é uma entrada
  do login dela vira `FalhaTransporte::ControleRedirecionadoParaOLogin`,
  com o subcódigo `sessao_encerrada_rode_ies_login` e `status_http` 302.
  É o subcódigo que a entrada por id e o link guardado já davam ao 302
  para o login: o mesmo fato tem o mesmo código, o agente que já trata
  esse subcódigo não muda, e a lista dos dois gatilhos fica igual.
- **subcodigo-novo-aceito-pelo-relogin:** um terceiro subcódigo
  (`redirecionamento_para_o_login_no_controle`). Mais preciso sobre a
  origem, mas a origem já está no `status_http` 302; criaria dois nomes
  para o mesmo fato e um terceiro gatilho a manter.
- **qualquer-302-do-controle:** rejeitada pelo proprietário. O Controle
  também responde 302 para ele mesmo (troca de unidade, nas cassettes da
  UnB, UFRJ e GO), e outro host nunca é fato verificável.

**Regra** ([`entrada_do_login.rs`](../../../src/sessao_login/entrada_do_login.rs),
chamada só em [`ler_tela_via`](../../../src/transporte/envio_post.rs)):
status exatamente 302; `Location` resolvido pelo mesmo intérprete do
login (`resolver_no_host`, que usa `sufixo_da_mesma_origem`: mesmo host,
`http://` no mesmo host conta, sem `.`/`..`); caminho exatamente
`/sip/login.php`, `<base>/modulos/goias/login-sei/MdGoiasLoginSEI.php`,
`<base>/modulos/goias/login-sei/login.php` ou `<base>/`; consulta livre,
salvo `msg=Link sem assinatura`. Fragmento, outro host, outra porta,
outro caminho, outro 3xx ou `Location` ausente: `redirecionamento_recusado`,
como antes. O 302 nunca é seguido: um pedido.

**Onde vale.** Todo GET do Controle por `ler_tela_via`: `unidade atual`,
`listar-processos`, `recebidos`, `gerados`, `colunas` (com e sem
`--definir`; a escrita só ganha o subcódigo, nunca o relogin),
`status --verificar-controle-web`, `processo buscar`, `documento buscar` e
o Controle de `processo NUP`/`documento DOC`. A entrada por id e o link
guardado de `processo NUP` e `documento DOC` já davam
`sessao_encerrada_rode_ies_login` ao 302 para `login.php` e não mudaram.
Os GETs seguintes de links publicados (nível de detalhe, tela da
pesquisa) e os POSTs mantêm o tratamento atual.

**Evidência.** As cassettes das 7 instalações não têm GET do Controle com
302 para o login; as formas das entradas vêm da cadeia de login
observada ([sessão e login](../../como-o-sei-funciona/sessao-login.md#302-do-controle-com-sessão-encerrada)).
RED: 5 dos 7 testes novos falharam com o shim (os 2 que passavam afirmam
o comportamento mantido: outros 302 recusados); GREEN: 1.310 testes da `ies`.
Live PEN4 em 05/11/2026: `unidade listar-processos` com sessão
desconhecida, 8 túneis = 1 + login 6 + releitura 1, `relogin: true`
([relatório](../../validacao-live/2026-11-05-relogin.md)).

**Limites.** Qual das entradas a PEN4 pôs no `Location` não foi
registrado (a consulta pode trazer `infra_url` com a URL assinada; nada
foi impresso). Vencimento natural continua NOT_RUN: se ele der um 302
para outro lugar, a regra não muda sem nova observação.
