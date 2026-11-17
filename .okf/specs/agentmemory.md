---
type: Spec
title: AgentMemory
description: Tipo OKF para memória operacional aditiva dos agentes do IES
---

# AgentMemory

Uma memória é um documento OKF **imutável por padrão** que transporta
contexto de uma rodada para a próxima. Ao encerrar **cada rodada**, o
agente cria uma memória breve. Pode combinar decisão, aprendizado,
dúvida, hipótese ou inspiração livre, inclusive originada no Cafezinho.
Não é instrução normativa e não substitui a `main`, contratos, issues
ou PRs. Evite relatórios de sessão, cópias de logs e duplicações.

## Frontmatter

- `type: AgentMemory`: único tipo para todos os agentes.
- `title`: assunto breve e específico.
- `description`: **resumo** suficiente para decidir se vale ler o corpo.
- `agent`: slug exato do autor (`raposa`, `coruja`, `tartaruga`,
  `gaviao`, `polvo`, `castor`, `texugo` ou `elefante`).
- `timestamp`: instante de criação em UTC, ISO 8601 com `Z`, **entre
  aspas**; não inventar data nem usar horário local sem offset.
- `tags`: lista YAML curta de assuntos, sem catálogo obrigatório.

O corpo é livre: registra o que o agente deseja reencontrar (inclusive
ideia criativa ou conversa sem aplicação imediata), sua fonte quando
houver e eventual próximo passo. **Diferencie observação, hipótese
e ficção**: um enigma inventado no Cafezinho pode inspirar, mas não
prova uma capacidade do IES nem cria uma obrigação no roadmap.
Se nada novo ocorreu, registre o ponto de retomada sem inventar fatos.

## Identidade e fixação

Local: `team/memories/<agent>/YYYYMMDDTHHMMSSZ-<slug>.md`.
A ordenação dos nomes dentro da pasta equivale à ordem cronológica.
Arquivos novos são commits diretos na `main`, sem PR; não reescreva
memórias antigas. Para corrigir um registro, acrescente outro e cite o
anterior. Não registre segredos, dados de sessão ou dados privados.

**Não existe `pinned` em AgentMemory.** Cada ficha `TeamMember`
controla suas memórias fixadas por meio de
`memorias_fixadas: [team/memories/<agent>/...]`. Fixar/desafixar
altera somente a ficha, não o registro histórico.

Ver [guia de memórias](../../team/memories/README.md).
