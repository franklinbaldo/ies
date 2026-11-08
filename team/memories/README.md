---
type: Guide
title: Memórias OKF dos agentes
description: Criar, recuperar e fixar memórias operacionais sem PR
---

# Memórias da equipe

Cada agente guarda memórias em `team/memories/<agente>/`. Cada rodada
termina com **uma memória nova**: é a ponte entre a sessão atual e a
próxima. O `description` serve de resumo; data UTC e tags ficam no
frontmatter. Não crie arquivos vazios só para inicializar pastas.

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

## Encerramento: trabalho → Cafezinho → memória → descanso

**Em toda rodada**, depois do trabalho e de ler a conversa na
[issue #388](https://github.com/franklinbaldo/ies/issues/388), o
agente registra **uma memória curta antes de encerrar** e a publica
por commit direto na `main`. Comentar no Cafezinho é opcional;
registrar a memória não é. O café não tem pauta nem obrigação de
gerar ideia útil.

A memória pode reunir, em texto livre, qualquer coisa que valha
retomar: o que funcionou ou falhou, próximo passo, pergunta em aberto,
hipótese revisável, experiência de outro agente, livro, filme,
conversa, brincadeira ou inspiração do Cafezinho. Se mencionar uma
conversa, prefira o link para o comentário original. Não converta
ficção em evidência técnica e não atribua fatos não verificados ao
código. **Se nada mudou**, registre apenas o ponto real de retomada
ou uma pergunta honesta; não fabrique um achado.

Prefira poucas linhas, com um resumo que permita encontrá-la pelo
OKF. Não transcreva a conversa inteira nem reproduza o handoff/CI.
Pode citar memórias de colegas quando isso conectar ideias; não é
preciso ler todas. A rodada seguinte recupera esta nota pelas cinco
memórias recentes do próprio agente, além das fixadas.

Exemplo de corpo, sem campos obrigatórios:

```markdown
# Para lembrar na próxima rodada

A verificação independente distingue um carimbo existente de um
empréstimo efetivamente realizado — analogia interessante da
[conversa no Cafezinho](https://github.com/franklinbaldo/ies/issues/388).
Isso é inspiração, não evidência de comportamento do IES.
Retomar a issue em andamento antes de escolher outra tarefa.
```

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
