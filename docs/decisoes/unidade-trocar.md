---
type: ImplementationDecision
title: Troca explícita da unidade da sessão pela tela Trocar Unidade, verificada pelo Controle da unidade nova
operation: unidade.trocar
implementation: partial
status: adopted-for-scope
scope: ies unidade trocar SIGLA [--confirmar] pela Web com sessão existente; prévia pela tela infra_trocar_unidade aberta do lnkInfraUnidade do Controle; com --confirmar, um POST derivado de frmInfraSelecaoUnidade com selInfraUnidades, os dois 302 observados até o Controle da unidade nova, a URL da sessão regravada e os links assinados da instalação apagados do índice local
selected_strategy: web-tela-de-troca-e-controle-novo
alternatives: web-tela-de-troca-e-controle-novo,web-troca-implicita-por-comando,web-link-guardado-da-outra-unidade,web-relogin-na-unidade,rest-wssei
evidence_level: acervo-publico-tres-instalacoes-testes-sinteticos-e-live-ufrj
live_validation: pass-ufrj-4-1-5-0511
---

# Troca de unidade

Contrato em [unidade trocar](../../contracts/cli/unidade-trocar.md);
observação em [troca de unidade](../como-o-sei-funciona/troca-de-unidade.md);
evidência por operação em [unidade.trocar](../../contracts/evidencias/unidade-trocar.md).
Aprovado pelo proprietário em 04/11/2026 (#205): os ensaios de
`processo enviar` (#173) deixavam processos abertos na unidade de destino
porque o IES não trocava de unidade.

## Alternativas consideradas

- **web-tela-de-troca-e-controle-novo** (escolhida): comando próprio,
  `ies unidade trocar SIGLA [--confirmar]`. A prévia lê a tela que o link
  da sigla abre; a escrita é o POST de `selecionarUnidade()`, e a prova é
  o Controle que os dois 302 entregam, cuja URL passa a ser a da sessão.
- **web-troca-implicita-por-comando:** `--unidade X` em qualquer comando
  trocaria antes de agir, como o `SEI_UNIDADE` do Todos.
- **web-link-guardado-da-outra-unidade:** abrir um link assinado guardado
  de outra unidade, que o SEI trata como troca ("Detectada troca de
  unidade").
- **web-relogin-na-unidade:** novo login e troca na sessão nova.
- **rest-wssei:** não há evidência de serviço REST na UFRJ para a troca.

## Comparação

- **Efeito para outros agentes.** A unidade é estado da sessão para todos
  que usam o cookie. A troca implícita mudaria a unidade de todos os
  agentes como efeito colateral de uma leitura; o proprietário decidiu que
  o IES nunca troca de unidade por conta própria. O comando próprio torna
  a troca uma escrita pedida, com prévia e `--confirmar`.
- **Link de outra unidade.** Na UnB o GET da URL do Controle de Unidade1
  com a sessão em Unidade2 respondeu 302 para o Controle de **Unidade2**
  (a sessão não voltou); o Todos mediu links do menu recusados com 302
  para o login. Não troca de unidade e arrisca a sessão: descartado.
- **Relogin.** Gastaria o login (4 a 6 requisições) e derrubaria a sessão
  dos outros agentes, sem evitar a tela de troca.
- **Custo.** Prévia 2 requisições (Controle, tela); escrita 5 (mais POST e
  dois GET dos 302). Não há leitura a mais: o último GET é o Controle da
  unidade nova, que verifica e dá a URL da sessão.
- **URL da sessão.** Os dois saltos levam `infra_hash` de 192
  hexadecimais (UnB, UFRJ, GO). Guardar o Controle do último salto exige
  que a URL da sessão aceite 192 além de 64; a alternativa, guardar um
  link de 64 do Controle novo, só existe com `reset=1` ou com
  `tipo_filtro`, que mudam o filtro da tela a cada GET de todos os
  comandos. Escolhida a URL que o servidor entregou, com a validação de
  `validar_url` ampliada só no comprimento do hash (64 ou 192).

## Decisão

Adotada **web-tela-de-troca-e-controle-novo**:

- Primitivo de formulário: uma forma nova de envio,
  `EnvioDoFormulario::SubmitPorScriptComCampoCriado(campo)` — o script da
  tela cria um oculto (`selInfraUnidades`) e chama `submit()`; o valor vem
  da operação e o campo sai no fim do corpo (`appendChild`). Publicado na
  tela com o mesmo nome é recusa. A `const` `TROCAR_UNIDADE` fica na
  operação (`src/unidade_trocar/tela.rs`), com
  `EfeitoAvaliado::AlteraEstadoDaSessao { UnidadeAtual }`.
- `Location` dos 302 pela regra compartilhada
  (`formulario_web::consulta_do_controlador`, que usa
  `sessao_login::redirecionamento::sufixo_da_mesma_origem`): `http://` no
  mesmo host aceito (GO), pedido sempre em `https://`. Espaço literal do
  `msg` vira `%20`, a serialização do navegador; qualquer outro byte fora
  de 33–126 é desvio (saída 1).
- Depois do POST não há reconciliação por outra URL: a URL guardada é da
  unidade anterior e não prova nada (e pode derrubar a sessão). Todo
  desvio sai com 1, sem novo pedido, com a etapa e o fato.
- Sessão: sob o lock do login, a URL do Controle novo é gravada (temporário
  0600 e `rename`) e o marcador de geração é atualizado com o mesmo
  cookie, para `ies login` reconhecer a sessão. O índice local apaga todos
  os links assinados da instalação (os da unidade anterior não abrem a
  mesma tela) e colhe o Controle novo, já chaveado pela unidade nova.
- Saída e contrato dizem que a troca vale para toda a sessão da conta
  (`escopo: sessao_da_conta`).

## Evidências verificáveis

- Acervo público: [replay](../validacao-replay/2026-11-05-unidade-trocar-acervo.md)
  das 22 trocas gravadas (UnB 8, UFRJ 8, GO 6) pelos mesmos leitores
  (`src/unidade_trocar/acervo.rs`, ignorado no CI).
- Testes sintéticos: `unidade_trocar::tests_tela` (link, tela, escolha,
  derivação), `unidade_trocar::tests` (sequências exatas, saídas 0/1/3,
  `http://`, 192 hexadecimais, espaços do `msg`), `formulario_web::tests_campo_criado`,
  `indice_local::tests_links` (links da instalação apagados),
  `unidade_trocar_cli::tests` (URL da sessão e marcador regravados, nada
  da unidade anterior reaproveitado) e
  [CLI](../../tests/cli_unidade_trocar.rs).
- RED: 23 testes falhando com shims (primitivo, transporte, índice, leitores, fluxo, sessão).
- Live **PASS** na [UFRJ 4.1.5](../validacao-live/2026-11-05-unidade-trocar.md): PR-P → PR-A → PR-P, prévia 2, troca 5 requisições, iguais aos túneis do relay.

## Limitações e revisão

Não observados: sigla repetida, lista paginada, 200 ao POST, `Location`
relativo e `msg` fora do ASCII (todos recusa ou saída 1). Sem PEN. Rever
se a tela publicar a troca por outro script, se o 302 mudar de forma ou se
o hash dos saltos mudar de comprimento.
