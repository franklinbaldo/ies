---
type: OperationEvidence
title: Validação sintética da inclusão e pendência live de Acompanhamento Especial
description: Resultados observados e lacunas de validação para o handoff da issue 254
operation: processo.acompanhar
---

# Relatório da issue #254 — 05/11/2026

- **UnB / prévia Web:** NOT_RUN; **0 requisições**.
- **Segunda instalação / prévia:** NOT_RUN; **0 requisições**.
- **Escrita de acompanhamento em treinamento:** NOT_RUN; **0 POSTs**.
- **Resíduo criado ou alterado no SEI:** nenhum.
- **Replay do acervo:** NOT_RUN. Índices consultados, incluindo
  `processo-acompanhar-unb`, `processo-acompanhar-go`,
  `grupo-acompanhamento-ciclo-unb`.
- **RED local:** NOT_RUN (0 testes executados, número de falhas
  não observado). Sandbox sem `rustc` ou `cargo`; DNS não resolve
  `github.com`, impedindo buscar dependências e instalar Rust.
- **Testes sintéticos publicados:** seleção exata, ambiguidade, URL
  de outro processo, tabela ausente, IDs repetidos, reconciliação
  de inclusão e retirada, derivação exata do formulário, fluxo GET/POST
  com transporte injetado, orçamento 5 GETs na prévia e 7
  requisições na inclusão (3 abertura + gerenciar + cadastro + POST
  + releitura), falha de rede no POST, falha de releitura e ausência
  do efeito. CI deve ser conferido no head final antes de aprovação.

## Limites

A inclusão com `--confirmar` já passa por prévia e derivação
do formulário servido, envia um único POST, sem retry nem seguir
redirect, e faz releitura do gerenciador para conferir o efeito;
na ausência de prova de inclusão sai com código 3. A retirada
confirmada permanece bloqueada antes de carregar a sessão.
**Esta execução não usou credenciais nem enviou requisição real**,
portanto não comprova compatibilidade live. A validação autorizada
continua pendente exclusivamente em **treinamento**, na ordem
prévia → único confirmar → releitura, registrando o resíduo.

## Segunda fatia — 🐢 Tartaruga: oferta de exclusão (#254)

- **Observação pré-RED:** fonte UnB em `Todos tests/test_processo_acompanhar_010.py`
  @323eb01: cada linha oferece `acaoExcluir(id, desc)`, que atribui
  `hdnInfraItemId`, muda a ação de `frmGerenciarAcompanhamento` para
  `acompanhamento_excluir` e submete o mesmo formulário.
- **RED:** PASS como observação de falha; CI [42118873078](https://github.com/franklinbaldo/ies/actions/runs/42118873078),
  **1 teste novo falhou** no scaffold (1.499 passaram, 15 ignorados preexistentes).
  Rustfmt e Clippy da versão RED também falharam; não são considerados
  gates verdes. **RED local:** NOT_RUN (sem Rust; DNS indisponível).
- **Validação sintética:** oferta da linha escolhida, função servida,
  formulário, campo oculto e identidade do destino; casos de ausência,
  desvio do processo e link de outra linha. O resultado final dos gates
  pertence ao CI do head mais recente, não ao RED.
- **Prévia ao vivo em treinamento:** NOT_RUN, **0 requisições**.
- **Confirmação ao vivo:** NOT_RUN, **0 POSTs**. A exclusão por
  `--deixar --confirmar` continua bloqueada antes da sessão.
- **Releitura ao vivo:** NOT_RUN, **0 GETs**. **Resíduo no SEI:** nenhum.
- **Replay do acervo:** NOT_RUN nesta fatia; os índices fornecem contagens,
  mas não substituem os corpos originais das cassettes.

## Regressões adicionais da segunda fatia

A revisão do comando de retirada passou a recusar o oculto `hdnInfraItemId`
quando seu `id` DOM não corresponde ao nome, quando o mesmo `id` aparece
fora do formulário (o `getElementById` escreveria em outro elemento) ou
quando o input está `disabled` e não seria enviado. Três testes sintéticos
negativos exercitam essas fronteiras. Outro teste positivo percorre a prévia
com transporte injetado e afirma **4 GETs exatos, 0 POSTs**. Nenhum teste
anterior foi ignorado ou enfraquecido.

- **RED inicial:** 1 falha observada no CI
  [42118873078](https://github.com/franklinbaldo/ies/actions/runs/42118873078).
  O RED separado dos três testes negativos adicionais ficou `NOT_RUN`;
  não foi possível executá-los localmente antes da correção por falta de Rust.
- **GREEN na bancada CI:** [42124400490](https://github.com/franklinbaldo/ies/actions/runs/42124400490),
  11 gates `PASS`, incluindo Rustfmt, Clippy, Rust, OKF e `wip-ignores`,
  com uma mescla real da `main` (sem publicação de escrita no SEI).
  O CI canônico do head final deve ser conferido separadamente.
- **Prévia autenticada, confirmação, releitura e replay de cassettes:**
  `NOT_RUN`; 0 GETs ao SEI, 0 POSTs ao SEI; resíduo: **nenhum**.
  `--deixar --confirmar` ainda recusa antes de ler sessão ou abrir rede.

## Terceira fatia — 🐢 Tartaruga: escopo real de `acaoExcluir`

- **Observação antes dos testes:** `Todos tests/test_processo_acompanhar_010.py`
  @323eb01 mostra o corpo `acaoExcluir(id,desc)`, o `if(confirm(...))`,
  o oculto `hdnInfraItemId`, a URL de exclusão e o `submit()`.
- **Regressão corrigida:** a validação anterior buscava instruções no
  `script` inteiro, mesmo fora da função. Agora delimita o corpo por
  chaves e strings e exige a forma servida, sem código adicional.
- **Testes sintéticos:** comandos deslocados para outra função, trechos
  fora da função, condição adicional inatingível e chave em texto de
  confirmação; não há teste pulado ou enfraquecido.
- **RED dos testes novos:** `NOT_RUN` como falha comprovada. O CI do
  commit de testes isolados foi cancelado ao chegar a implementação;
  o RED histórico da fatia anterior segue em 1 teste falhando.
- **Live:** prévia `NOT_RUN` (0 GET); confirmação `NOT_RUN` (0 POST);
  releitura `NOT_RUN` (0 GET). Replay do acervo `NOT_RUN`.
  **Resíduo no SEI: nenhum.** Nenhuma instalação foi acessada.
- **Segurança:** `--deixar --confirmar` permanece bloqueado antes de
  carregar a sessão; falta derivar o POST único e validar a conferência.
