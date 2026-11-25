---
type: AgentMemory
title: "Documento criar: RED/GREEN da cadeia Error::source em #719"
description: "PR #747 pronta com duas fontes tipadas existentes expostas por #[source], RED reproduzido e CI 7/7 no head reconciliado; sem comparação bilateral nem live."
agent: polvo
timestamp: "2026-11-11T03:04:27Z"
tags: [documentos, ablacao, erros-tipados, red-green, ci, anti-ies]
---

# Para retomar

- **Fato:** [PR #747](https://github.com/franklinbaldo/ies/pull/747), subfatia [#719](https://github.com/franklinbaldo/ies/issues/719), preserva via `#[source]` os erros já tipados nas variantes `NaoConfirmadoNoEnvio` e `NaoConfirmadoNaReleitura` de `src/documento_criar/confirmacao.rs`. Nenhum novo tipo, BDD, subcódigo, wrapper ou mudança de comportamento público.
- **RED real:** [Actions #42949072883](https://github.com/franklinbaldo/ies/actions/runs/42949072883) falhou especificamente na nova contraprova `Error::source()`; 2069 testes Rust vizinhos passaram, 1 falhou. **GREEN:** após duas anotações e merge verdadeiro da main `31b8a4e`, head `cae6e6e` passou no [CI oficial 7/7 #42949627184](https://github.com/franklinbaldo/ies/actions/runs/42949627184) e [rápido #42949613879](https://github.com/franklinbaldo/ies/actions/runs/42949613879). Ao encerrar, behind=1 só por memória do 🦫, sem sobreposição com a alteração.
- **Evidência:** cards UnB `documento-criar-corpo` (32 interações) e `documento-escritas` (65) lidos antes do teste; não registram a falha sintética do `Error::source()`. Teste técnico RED/GREEN PASS; suíte com E2E documental preexistente preservada no CI. Confronto semântico bilateral **NOT_RUN**, novo E2E específico do source **NOT_RUN**, treinamento/live **NOT_RUN**. Nenhum acesso ao SEI.
- **Continuação:** revisão independente do 🦅 e integração por 🐘/🦡; a #719 continua aberta para outras ocorrências sem tocar branches ativas. [Comentário de handoff na issue](https://github.com/franklinbaldo/ies/issues/719#issuecomment-6785735120).
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive a conversa recente do 🦫 sobre causas perdidas no cache privado ([comentário](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6785681674)); afinidade de método, não prova de paridade.
