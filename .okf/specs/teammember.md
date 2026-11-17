---
type: Spec
title: TeamMember
description: Tipo OKF compartilhado por todas as fichas dos agentes do IES
---

# TeamMember

Representa **um membro da equipe de agentes do IES**, independentemente
de sua função, especialidade ou identidade. Todas as fichas em
[`team/`](../../team/) têm `type: TeamMember`; nenhum membro precisa de
tipo OKF próprio.

## Campos do frontmatter

- `title`: nome público do agente, incluindo seu emoji/avatar.
- `description`: resumo da ficha e de sua finalidade.
- `avatar`: emoji identificador da rotina (🦊, 🦉, 🐢, 🐙, 🦫, 🦡, 🦅 ou 🐘).
- `especialidade`: domínio de responsabilidade prioritária.
- `frequencia`: horário/cadência da rotina, em forma legível.
- `memorias_fixadas`: lista de paths OKF relativos à raiz, das
  memórias `AgentMemory` que o agente quer ler sempre. A lista vazia
  significa que nada está fixado; fixar/desafixar só edita a ficha.

O esquema SQL é [`teammember.schema.sql`](teammember.schema.sql),
gerado por `uvx --from okf-parser==0.48.2 okf-parser init team
--spec-template '.okf/specs/{slug}.md' --write --infer-schema`.

## Semântica da equipe

O tipo apenas classifica o documento e seus metadados. Permissões,
reservas, prioridades, revisão, integração e segurança seguem
[`AGENTS.md`](../../AGENTS.md); cada ficha explica sua missão e limites.
O avatar distingue pessoas **dentro do mesmo tipo**, não cria tipos novos.
