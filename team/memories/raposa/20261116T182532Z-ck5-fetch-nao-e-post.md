---
type: AgentMemory
title: "🦊 #851: fetch no HTML CKEditor 5 não prova POST de salvamento"
description: "Nos GETs originais GO/PEN3/PEN4, o fetch encontrado pertence à rotina de erro; os pedidos editor_salvar_conteudo não guardam body nem headers."
agent: raposa
timestamp: "2026-11-16T18:25:32Z"
tags: [ies, documento-salvar, ckeditor5, cassette, evidencia-negativa]
---

# Retomar a prova correta

No Ifro, sem rede SEI, conferi SHA-1 dos ZIPs e SHA-256 LF dos membros `documento-escritas` de GO, PEN3 e PEN4: **3/3 de cada íntegros**. Seus nove GETs `editor_montar` incluem `new InfraEditor(...)` e exatamente uma chamada textual `fetch(` por tela; a chamada localizada está na rotina minificada de relato de erro (referências a `navigator.userAgent`, `URLSearchParams` e detalhes de erro), **não** constitui captura de envio de conteúdo. Não encontrei definição textual de `class/function InfraEditor` no HTML por esses padrões; isso não prova ausência de implementação carregada de outro recurso.

Os seis POSTs `editor_salvar_conteudo` nessas três cassettes só registram `method`, `operation`, `operation_parameter` e `pathname` na requisição. **0/6** preservam headers, JSON, form fields ou body. O total histórico das sete cassettes continua **14 POSTs sem payload**, sem nova inferência de protocolo. O modelo de POST do Todos permanece oráculo candidato, não tráfego capturado.

A guarda de `remoto::executar` recusa origens não `sei.test` **antes** do GET de editor. Não promover o leitor CKEditor5 WIP a produção sem consumidor funcional nem publicar outra recusa redundante. Nenhum código/PR/branch novo, nenhum GET/POST SEI. Parser Rust WIP e E2E novo/CI final: **NOT_RUN**; teste de formatação no contêiner Rust 1.91 não executou porque `cargo-fmt` não estava instalado. Próxima prova útil exige consumer real ou captura do protocolo em ensaio separado e autorizado, com avaliação de efeitos GET, documento não assinado, POST único sem retry e releitura independente. Handoff: [#851](https://github.com/franklinbaldo/ies/issues/851).
