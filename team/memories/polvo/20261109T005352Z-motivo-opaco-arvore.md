---
type: AgentMemory
title: "Motivo opaco do cancelamento: RED encontrou trim na releitura"
description: "RED do BDD-273-02 mostrou saída 3 para motivo com espaços; a árvore eliminava espaços via trim, correção na PR #523 aguarda CI completo."
agent: polvo
timestamp: "2026-11-09T00:53:52Z"
tags: [documento, cancelamento, motivo, anti-ies, red-green]
---

# Retomar

- [PR #523](https://github.com/franklinbaldo/ies/pull/523) acrescenta regressão da CLI real e SQLite para motivo com espaços, quebra de linha, Unicode e caracteres especiais, sem novos BDDs.
- [CI RED #42542232247](https://github.com/franklinbaldo/ies/actions/runs/42542232247): 17/18 testes do módulo passaram; o novo teste saiu 3, não 0. Causa identificada: `arvore_processo/natureza.rs` usava `str::trim` no motivo do cancelamento, apagando espaços que o contrato declara opacos. Correção pontual e expectativa unitária atualizadas na branch; **GREEN ainda requer CI**.
- [Inspeção Actions #42542005164](https://github.com/franklinbaldo/ies/actions/runs/42542005164): cassettes GO/UFRJ com hash válido, 2 GET+1 POST de `documento_cancelar` por instalação. Isso **não** é confronto semântico independente nem teste live.
- [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388): a ideia narrativa de testemunhas independentes me lembra distinguir o texto persistido do texto efetivamente lido. Analogia, não evidência SEI.
- Continuar a mesma PR até GREEN e revisão, sem produção, credenciais, novos subcódigos ou escrita externa. O conflito de `--motivo ""` entre checklist e contrato permanece para conciliação pelo 🐘.
