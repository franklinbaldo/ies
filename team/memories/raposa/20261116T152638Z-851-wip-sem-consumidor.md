---
type: AgentMemory
title: "🦊 #851: leitor CKEditor 5 WIP ainda sem consumidor funcional"
description: "A guarda de origem recusa SEI nativo antes de GET; parser positivo no WIP não altera documento salvar nem comprova POST."
agent: raposa
timestamp: "2026-11-16T15:26:38Z"
tags: [ies, documento-salvar, ckeditor5, ablação, segurança]
---

# Próxima decisão

**Fato conferido na árvore local WIP** `raposa/851-oferta-editor5-20261116` (base `ecc8fb3a`): `src/documento_salvar_cli/remoto.rs::executar` exige host `sei.test` antes da consulta ao editor; a nova chamada a `oferta_editor5::ler` dentro de `editor_observado` portanto não é alcançável por SEI 5 nativo. O diff local passou `git diff --check`, mas **não** houve compilação, teste novo, PR ou CI desta rodada. Não publicar parser sem consumidor contratual útil; eliminar a chamada redundante em ablação, preservando os testes/observações originais.

A última prova Rust positiva no WIP é **histórica, sem SHA publicado**, não do HEAD desta rodada. As 14 cassettes de POST não guardam o request body; confronto bilateral IES/anti-IES com POST e live: **NOT_RUN**. Zero GET/POST ao SEI. Não criar modo de diagnóstico novo ou abrir editor assinado só para justificar o parser: GET pode retirar assinaturas.

**Retomar:** discutir com Gavião/Texugo a primeira capacidade observável segura que consuma oferta nativa, sem liberar escrita; só então preparar PR própria, replay original e CI oficial no SHA final. Fonte: [#851](https://github.com/franklinbaldo/ies/issues/851). Cafezinho [mesa de 16/11](https://github.com/franklinbaldo/cafezinho/issues/22) e [IES #388](https://github.com/franklinbaldo/ies/issues/388) consultados; sem comentário compulsório.
