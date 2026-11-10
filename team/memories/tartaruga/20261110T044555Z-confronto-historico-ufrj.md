---
type: AgentMemory
title: "BDD-60-05: confronto independente do histórico UFRJ"
description: "PR #639 compara histórico e identidade da cassette UFRJ ao anti-IES HTTP/SQLite; CI 7/7 no SHA 993a85fa; corpos não capturados e live não testado."
agent: tartaruga
timestamp: "2026-11-10T04:45:55Z"
tags: [bdd60, anti-ies, ufrj, cassette-first, historico, sqlite, navegacao]
---

# Para retomar

**Fato:** a [PR #639](https://github.com/franklinbaldo/ies/pull/639) está Ready com diff restrito a um replay Rust e duas linhas no gate existente. A inspeção da [cassette UFRJ #42757194325](https://github.com/franklinbaldo/ies/actions/runs/42757194325) confirmou SHA-1/SHA-256 e 29 interações. O replay original UFRJ extrai identidade DOC/NUP da árvore e versões, autor, unidade e data do GET histórico, projeta por SQLite/HTTP e compara com o leitor real do IES, incluindo 404 para outro processo, POST 405 e preservação das revisões. Mantido o teste GO anterior. [CI completo #42757393364](https://github.com/franklinbaldo/ies/actions/runs/42757393364) **7/7 PASS**, [rápido #42757368603](https://github.com/franklinbaldo/ies/actions/runs/42757368603) PASS no head `993a85fa`. Não houve RED funcional, E2E CLI novo nem alteração produtiva.

**Três níveis:** CLI ↔ anti-IES ↔ SQLite PASS sintético nas regressões existentes; cassette UFRJ ↔ IES/anti-IES PASS parcial novo; treinamento/live/REA NOT_RUN. A cassette UFRJ não registra `documento_visualizar`: `<p></p>` é somente estrutura, sem alegação de corpo observado. Sem produção, credenciais, subcódigos ou TLS alterado.

**Coordenação:** [handoff #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6759234623); [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6759237202) visitado e comentado, reforçando a distinção entre não observado e observado vazio. Próxima rodada: revisão 🦅, integração 🐘/🦡 por merge verdadeiro e revalidação caso o head avance. Esta memória na `main` pode deixar a PR um commit atrás; não confundir CI anterior com validação do novo SHA.