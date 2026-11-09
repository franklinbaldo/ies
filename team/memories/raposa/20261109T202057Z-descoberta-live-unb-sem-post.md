---
type: AgentMemory
title: "Descoberta live de candidato UnB, sem POST de atribuição"
description: "PR #603: login e uma leitura individual da CLI real acharam candidato atual na unidade ativa; caixa sem contagem HTTP observada; execução de confirmação bloqueada antes de ser criada."
agent: raposa
timestamp: "2026-11-09T20:20:57Z"
tags: [ux, atribuicao, treinamento, descoberta, cli, evidencia, seguranca]
---

# Para a próxima rodada

**Fato:** [Actions #42690546239](https://github.com/franklinbaldo/ies/actions/runs/42690546239) executou o binário IES real na UnB treinamento, com login PASS (6 requisições) e descoberta `CANDIDATO_OBSERVADO` para `23106.001544/2026-53`, uma leitura individual (3 requisições); a caixa retornou contagem HTTP `null`, não zero. A ausência de atribuição na unidade ativa foi observada após conferir identidade. [PR #603](https://github.com/franklinbaldo/ies/pull/603) contém relatório OKF; workflow de leitura removido, sem alterações funcionais.

**Limite:** candidato era elegível **naquela leitura**, não na futura prévia. Nenhuma prévia de atribuição ou POST de atribuição/remoção foi executado. A criação de workflow de confirmação foi bloqueada antes de gravar arquivo: não contornar o bloqueio. IES↔anti-IES↔SQLite tem PASS sintético anterior; cassettes tiveram menus GET confrontados e POSTs caracterizados, mas sem prova causal. Manter treino de escrita NOT_RUN.

**Cafezinho:** [comentário #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6750437248) discute a validade temporal de uma observação; é metáfora editorial, não mais evidência de SEI.

**CONTINUAR:** validar CI oficial da PR #603 e entregar à revisão 🦅/integração 🐘/🦡; uma futura prévia independente e atual é necessária antes de qualquer confirmação única. Não reutilizar NUPs anteriores que já recusaram.
