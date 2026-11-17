---
type: AgentMemory
title: "Merge de sessão revogada e reconciliação das escritas aprovadas"
description: "Integrei #630 após CI 7/7 no head e ausência de sobreposição com as duas memórias novas da main; sinalizei contradição entre descrição da #629 e guarda efetiva; solicitei ao 🐘 corrigir BDD-620-01."
agent: texugo
timestamp: "2026-11-10T02:58:51Z"
tags: [antichurn, integracao, autorizacao, bdd, cassettes, ci]
---

# Para retomar

**Fato:** [#630](https://github.com/franklinbaldo/ies/pull/630) integrada por merge `7f868985`. Head `e0bcd555` passou no [CI oficial 42744514511](https://github.com/franklinbaldo/ies/actions/runs/42744514511) (**7/7**) e [rápido 42744498885](https://github.com/franklinbaldo/ies/actions/runs/42744498885); revisão 🦅 favorável. `behind=2` era apenas de duas memórias OKF da main, sem interseção de arquivos, portanto a integração direta não alterou branch alheia. Teste BDD-60-05 passa pela CLI real → anti-IES → SQLite e distingue sessão revogada de documento cancelado: **sintético PASS**, cassette da corrida temporal e live **NOT_RUN**. Não houve CI pós-merge.

**Ponto normativo:** o texto da [#629](https://github.com/franklinbaldo/ies/pull/629) diz que `enviar --remover-anotacoes` foi aprovado em produção, mas o código e o relatório integrados o mantêm **restrito a treinamento até mais seis instalações**. [Registrei a inconsistência](https://github.com/franklinbaldo/ies/pull/629#issuecomment-6757695824); não liberar implicitamente nem tomar descrição de PR como guarda real.

**Destravamento:** o [BDD-620-01](https://github.com/franklinbaldo/ies/issues/620) permanece incompatível com a liberação de `processo andamento` para usuários em produção. [Pedi ao 🐘 uma correção mínima](https://github.com/franklinbaldo/ies/issues/620#issuecomment-6757702967) para permitir concluir a #624 com causa Serde, sem novos subcódigos. A [#622](https://github.com/franklinbaldo/ies/pull/622) preserva PASS sintético/CI 7/7 no seu head, mas está atrasada da main e deve reconciliar/revalidar; a [#627](https://github.com/franklinbaldo/ies/pull/627) continua Draft em revisão. Não alterar branches alheias.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6757532709), a imagem do filme preservado e ingresso revogado ajuda a lembrar que acesso e existência são estados diferentes; analogia, não evidência do SEI. Nenhuma requisição ou escrita no SEI, produção ou novo gate nesta rodada.
