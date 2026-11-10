---
type: AgentMemory
title: "Confronto da pesquisa documental GO e UFRJ aguardando prova do runner"
description: "PR #669 reutiliza o confronto de navegação UnB com cassettes originais GO/UFRJ e o acervo ZIP SHA-verificado; Actions ainda sem resultado observado."
agent: tartaruga
timestamp: "2026-11-10T13:39:49Z"
tags: [bdd60, cassette-first, navegacao, anti-ies, sqlite, ci]
---

# Para retomar

**Fato:** a [PR #663](https://github.com/franklinbaldo/ies/pull/663) já foi integrada. Abri a [PR #669](https://github.com/franklinbaldo/ies/pull/669) da `main` para ampliar o confronto da seleção `procedimento_trabalhar` → `iframe#ifrArvore` → `procedimento_visualizar` às cassettes `documento-numero-e-busca` de Goiás e UFRJ, sem duplicar o teste UnB. Identidades e números vêm de cada gravação; somente os campos ausentes usam sentinelas estruturais. A mesma suíte HTTP/SQLite verifica respostas positivas e recusas sintéticas. Incluí os dois ZIPs em `cassettes/retornos-zips.txt` para aproveitar o cache e verificação SHA-1 do CI; membros JSON são verificados pelo replay Rust.

**Limite verificável:** os jobs da [inspeção de cassettes #42825811770](https://github.com/franklinbaldo/ies/actions/runs/42825811770) e do [CI completo #42826478236](https://github.com/franklinbaldo/ies/actions/runs/42826478236) ainda estavam na fila ao escrever esta memória. **Não declarar PASS nem paridade GO/UFRJ sem logs e gates do head final.** Inspeção estrutural também não equivale a confronto semântico. CLI Rust/TLS somente regressões já existentes; treinamento/live/REA NOT_RUN.

**Próxima rodada:** retomar #669 antes de nova issue; conferir o replay de cada instalação, tratar qualquer divergência real sem inventar dados, revalidar SHA sincronizado e só então entregar à revisão 🦅. O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) foi visitado; a distinção entre vazio intencional e ausência observada continua útil como analogia, não como evidência externa.
