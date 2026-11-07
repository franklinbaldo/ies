---
type: Guide
title: Memórias OKF dos agentes
description: Criar, recuperar e fixar memórias operacionais sem PR
---

# Memórias da equipe

Cada agente guarda memórias em `team/memories/<agente>/`. Um arquivo
é uma memória independente. O `description` serve de resumo; a data
UTC e as tags ficam no frontmatter. Não crie arquivos vazios só para
inicializar pastas.

## Exemplo de uma nova memória

Nome ilustrativo: `team/memories/raposa/20261107T203000Z-previa-cli.md`.

```markdown
---
type: AgentMemory
title: "Prévia da CLI: decisão observada"
description: "Resumo objetivo da decisão e da razão de lembrá-la."
agent: raposa
timestamp: "2026-11-07T20:30:00Z"
tags: [cli, previa]
---

# O que lembrar

Registre apenas fatos verificáveis ou hipóteses explicitamente marcadas,
com links para a issue, PR ou contrato quando relevante.
```

## Recuperação no início da rodada

1. Leia `AGENTS.md` e sua ficha `team/<agente>.md`.
2. Abra **todos os caminhos** presentes na lista `memorias_fixadas` da
   ficha; listas vazias não exigem nenhuma ação.
3. Leia até **cinco memórias mais recentes** de sua pasta, pela ordem
   decrescente do timestamp no nome; não releia arquivos já fixados.
4. Se precisar de assunto antigo, pesquise no OKF em vez de carregar
   a pasta inteira em todo ciclo.

Exemplo de consulta das memórias recentes da Raposa:

```sh
OKF="uvx --from okf-parser==0.48.2 okf-parser"
$OKF sql . "SELECT path, title, description FROM concepts WHERE concept_type = 'AgentMemory' AND path LIKE 'team/memories/raposa/%' ORDER BY path DESC LIMIT 5"
$OKF search . "prévia" --type AgentMemory --context 1
$OKF check . --require-spec .okf/specs/{slug}.md --normative-spec
```

A ferramenta facilita inventário, busca e validação; a leitura dos
arquivos Markdown e do frontmatter é a fonte primária da memória.

## Fixar e desafixar

Na ficha do agente, em vez de alterar a memória:

```yaml
memorias_fixadas:
  - team/memories/raposa/20261107T203000Z-previa-cli.md
```

Para desafixar, remova o caminho. A memória continua registrada e
pesquisável. Cada agente modifica apenas sua própria ficha e pasta,
por **commit direto** na `main`, sem PR, com seu avatar no começo
da mensagem. Não misture memória com alteração de código.

Memórias não substituem estado atualizado do GitHub e não justificam
escrita no SEI nem mudança dos limites de segurança.
