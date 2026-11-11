---
type: AgentMemory
title: "Proprietário priorizou ablação de antipatterns sobre novas capacidades"
description: "Roadmap #56 recebeu prioridade explícita de qualidade; #716 e famílias #717–#723 coordenam a eliminação com testes, não a criação de novas camadas de erro."
agent: elefante
timestamp: "2026-11-11T00:45:45Z"
tags: [prioridade, antipatterns, qualidade, ablação, rust, esteira]
---

# Ponto de retomada

- **Decisão do proprietário:** priorizar **identificar, falsificar e remover antipatterns de código**; novas capacidades passam a segundo plano. **Não** transformar a decisão em multiplicação de enums/estruturas de `Causa` ou hardening especulativo.
- **Providência material:** seção prioritária adicionada ao [roadmap #56](https://github.com/franklinbaldo/ies/issues/56); [comentário de coordenação](https://github.com/franklinbaldo/ies/issues/716#issuecomment-6783508005) na issue-mãe.
- **Sequência inicial:** #717 (erro unitário sem informação), #718 (`.ok()` mascarando falha pós-escrita), #719 (`map_err(|_|)` em pós-POST e I/O), #720 (ramos/guards que achatam resultados), #723 (nomes errados), #721 (supressões), #722 (defaults silenciosos). Não substituir padrões legítimos de domínio.
- **Primeira contraprova candidata:** `src/processo_enviar/fluxo.rs::observar_anotacao_depois` pode colapsar falha de leitura e estado não observado em `None`. Exigir teste antes de alterar e preservar contrato de erro sem novos subcódigos.
- **Cafezinho:** issue #388 visitada, com discussão sobre identidade, informação parcial, idempotência e fronteiras das provas. Nenhuma nova evidência live foi executada nesta rodada.
- **Próximo ciclo:** verificar quais sub-issues já receberam PRs e priorizar integração de ablações reais com CI completo e branch sincronizada; evitar conflitos com trabalho ativo. Se o contrato normativo de prioridade ainda disser "capacidades primeiro", considerar alteração mínima sem duplicar documentação.
