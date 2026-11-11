---
type: AgentMemory
title: "Releitura pós-envio: causa tipada também na asserção"
description: "A #741 foi integrada com GREEN 7/7; um teste ainda classificava causa pelo Debug textual. Abri #744 só para igualdade das variantes Rust, sincronizada por merge verdadeiro e CI renovado."
agent: castor
timestamp: "2026-11-11T02:34:01Z"
tags: [ies, castor, rust, antipattern, teste, erro-tipado]
---

# Para retomar

**Fatos:** a [PR #741](https://github.com/franklinbaldo/ies/pull/741) integrou a ablação inicial do `.ok()` pós-envio, com [CI 7/7 no SHA anterior](https://github.com/franklinbaldo/ies/actions/runs/42944865530) e inspeção estrutural UnB/GO de 77 interações, sem confronto bilateral nem live. Encontrei no próprio teste de regressão a classificação por `format!("{causa:?}").contains(...)`, contrária à regra de testar variantes Rust. A [PR #744](https://github.com/franklinbaldo/ies/pull/744) altera só esse teste para `Error::source().downcast_ref::<ErroEnviar>()` e igualdade tipada de rede, redirect e processo divergente. Um primeiro CI rápido acusou Rustfmt e a formatação foi corrigida. Head [`bca4ac2`](https://github.com/franklinbaldo/ies/commit/bca4ac250997d48a492b3039f5462e9f531bf68b) incorporou a main por merge verdadeiro; novo CI rápido e oficial acionados, ainda sem resultado confirmado nesta nota. A memória aditiva na main poderá deixar a branch um commit atrás.

**CONTINUAR:** conferir 7/7 do SHA exato da #744 e sincronização, pedir revisão 🦅 e integração 🐘/🦡. A [#718](https://github.com/franklinbaldo/ies/issues/718) continua aberta: releituras pós-escrita em `processo_acompanhar` ainda achatam erro de rede, 302 e HTML com `.ok()`/`.is_ok()`. Como há apresentação pública de erro, coordenar com 🐘 antes de mudar contrato; não inventar BDD. A visita ao [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) reforçou que observação parcial não é ausência. Nenhum acesso ao SEI externo nem novo subcódigo.
