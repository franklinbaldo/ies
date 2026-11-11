---
type: AgentMemory
title: "Replay por PR, sem contagem de requests como critério"
description: "Proprietário determinou replay de ZIPs no GitHub Actions por PR; rotina Texugo atualizada, PR #780 introduz execução do acervo e corrigiu leitor que tentava abrir arquivo como diretório."
agent: texugo
timestamp: "2026-11-11T11:01:25Z"
tags: [cassette-e2e, github-actions, antichurn, aceite-pr]
---

# Retomar

Fato: o proprietário determinou que a PR só seja integrada depois de o GitHub Actions executar os testes de cassettes aplicáveis no head final, sem Jatobá e sem utilizar contagem fixa de GET/POST como sinal de compatibilidade. O objetivo é que o anti-IES reproduza o comportamento observável do SEI gravado e que a CLI real do IES o reconheça. A inspeção estrutural dos ZIPs sozinha não comprova paridade.

A automação horária 🦡 foi alterada com sucesso. Abri a [PR #780](https://github.com/franklinbaldo/ies/pull/780) para aproveitar o job existente do CI e executar os replays Rust do acervo após verificar ZIPs/membros. O primeiro CI desse novo fluxo baixou e conferiu **495 cards e 29 ZIPs**, mas revelou RED real em 11 testes: `src/cassette/acervo.rs` tentava `read_dir` de arquivos da raiz de `cassettes/`. Corrigido no head `a71024c`, agora somente diretórios de instalação são percorridos; o CI do novo head foi disparado e seu resultado final ainda não estava comprovado na criação desta memória.

Não integrar a #780 até que o novo replay e os demais gates estejam PASS. Mesmo com PASS, não confundir execução dos testes existentes com cobertura semântica de todas as transições dos 495 cards. Hipóteses não capturadas permanecem identificadas como sintéticas. Cafezinho #388 visitado nesta rodada, sem comentário obrigatório.
