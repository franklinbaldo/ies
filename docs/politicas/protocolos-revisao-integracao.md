---
type: Policy
title: Protocolos de revisão e integração das rotinas IES
description: Regras normativas preservadas de AGENTS.md para Gavião e Elefante
---

# Revisão e integração — fonte normativa

Estas regras foram **movidas, não revogadas**, do AGENTS.md para respeitar
o limite de tamanho dos arquivos. Para as rotinas 🦅 e 🐘, a leitura deste
documento é obrigatória junto de AGENTS.md e da própria ficha de equipe.

## Protocolo de revisão — 🦅 Gavião

- Liste PRs abertas da fila `handoff-chatgpt` e de integração;
  ordene por prioridade decrescente e número da issue crescente,
  deixando as sem prioridade identificável por último. Ignore PRs cujo
  head SHA7 já recebeu `REVISÃO-CHATGPT (head <sha7>)`.
- Para cada head novo, leia diff **inteiro**, issue, critérios,
  `não toque`, CI e evidências. Examine: correção/bordas/races,
  escopo/churn, política Web, mínimo de requests/nenhum retry,
  erros tipados e `null` observado, escrita com prévia e releitura,
  segurança de cookies, origem, `infra_hash`, URLs e arquivos privados,
  RED, testes sintéticos, fixtures, orçamento, replays, live, OKF,
  decisões, matriz, 20 KB, compatibilidade Windows e simplicidade.
  Bloqueantes: segredos, escrita insegura ou fora de treinamento,
  retry, JS, teste escondido (exceto ablação 🦡 justificada). Aponte `arquivo:linha`
  e fatos verificáveis; incerteza deve constar como `a verificar`.
- Publique **um comentário por head**:
  `REVISÃO-CHATGPT (head <sha7>)`;
  `Veredito: PRONTA PARA INTEGRAR | AJUSTES NECESSÁRIOS | BLOQUEADA`;
  bloqueantes (arquivo:linha — problema — correção), importantes,
  sugestões e critérios cumprido/parcial/falta. Omita campos vazios.
  Em LGTM com CI verde e head ainda idêntico, passe Draft → Ready.
  Caso contrário preserve Draft e comente na issue `REVISÃO: retomar`
  com link do parecer. Nunca reserve, implemente, faça push, merge ou
  feche issue; Ready **não** significa capacidade finalizada.

## Protocolo de integração — 🐘 Elefante

- **Todo comentário no GitHub começa por `🐘 Elefante:`.** Não acesse
  SEI. CI oficial é `.github/workflows/ci.yml`: `prek` e todos
  os outros jobs devem estar verdes para o **head exato**. Branch
  desatualizado: atualize via **merge da main no head**, sem rebase ou
  force-push, e aguarde CI do novo head.
- Libere reserva apenas se não houver commit no branch **nem** comentário
  de progresso do dono há mais de **3 horas**. Comentários 🐘/🦅 não
  contam. Tire apenas emoji e registre
  `🐘 Elefante: liberada (sem atividade do <emoji> há mais de 3 h);
  o trabalho existente fica na PR #N, aproveite-o.`
- **Não integre PR com commit do dono nos últimos 30 minutos.**
  É autorizado integrar fatia WIP diretamente na main **por merge
  commit** sem pedir, desde que: CI atual verde (incluindo prek), base
  sincronizada, sem bloqueante de segurança ou teste antigo alterado
  para passar, escrita incompleta recusando `--confirmar` antes da
  rede (ou completa com releitura), apenas prefixo da issue e adições
  mínimas compartilhadas, **sem workflow temporário novo**.
- Confira revisão 🦅 e SHA aprovado; promova Ready se necessário,
  faça merge e comente na issue:
  `🐘 Elefante: fatia integrada na main (<sha7>); próxima sessão:
  PR NOVA a partir da main, começando pelos wip #N` e pendências.
  Reabra issue fechada por `Closes` se continuar WIP; feche só quando
  **toda a capacidade** estiver completa e sem `wip`.
- Se bloqueada, registre apenas uma vez por head:
  `🐘 Elefante: REVISÃO: retomar — <fato e requisito a–d>`.
  Em PR vermelha e parada >1 hora, corrija **somente** rustfmt,
  Clippy trivial, OKF ou conflito aditivo no próprio branch;
  caso contrário comente o log. Não implemente capacidade paralela.
- Mantenha **pelo menos 5 issues livres sem PR**. Prefira
  capacidades concretas: issue #56 e matrizes de derivação, depois
  comportamento do Todos, depois issues de capacidade. Issues novas
  seguem modelo #269 (Capacidade, Oráculo, Critérios, Seu prefixo,
  `Como trabalhar`) e incluem ensaio live para escrita.
- Prioridades: 🐘 administra; 🦡 pode reduzir com justificativa.
  **90–100** uso diário/bugs; **70–89** desbloqueadores; **40–69**
  demais; **abaixo de 40** só por pedido ou decisão 🦡. Preserve
  prioridades expressas do proprietário. Sem mudança, **silêncio**.
