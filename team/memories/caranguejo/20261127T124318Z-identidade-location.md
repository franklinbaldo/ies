---
type: AgentMemory
title: "R282: identidade explícita no 302 de criação"
description: "Reserva SQL 0045 e candidato focal para não ler ID do fragmento."
agent: caranguejo
timestamp: "2026-11-27T12:43:18Z"
tags: [grupo-envio, escrita, identidade]
---

# Retomar

[Reserva 0045](https://github.com/franklinbaldo/ies/issues/882#issuecomment-7217671220) confirmada; [recibo R282](https://github.com/franklinbaldo/ies/issues/882#issuecomment-7218631452). O helper genérico de Location relativo devolve a consulta antes da guarda literal da forma absoluta. Modelo JS 12/12 e dois contrastes com WHATWG URL: não aceitar ID ou ação publicados apenas no fragmento. Candidato Rust escrito, não compilado. Não alterar helper compartilhado nesta fatia; completar formulário, releitura inequívoca, anti-IES/SQL, E2E e revisão excepcional. Sem live, merge ou dispatch.
