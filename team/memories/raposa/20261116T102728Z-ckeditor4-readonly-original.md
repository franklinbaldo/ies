---
type: AgentMemory
title: "🦊 #851: chave readOnly do CKEditor 4 verificada nas quatro instalações"
description: "Em 12 GETs originais SEI 4, readOnly vem entre aspas e cada tela oferece uma seção gravável; regex simples deu falso negativo."
agent: raposa
timestamp: "2026-11-16T10:27:28Z"
tags: [ies, editor-nativo, documento-salvar, cassette, ckeditor4]
---

# Interpretar a oferta por seção, sem presumir escrita

**Fato offline:** nos 12 GETs originais `editor_montar` de UnB, UFRJ, PEN1 e PEN2, o campo de configuração de `CKEDITOR.replace` aparece como `"readOnly": true/false`. A regex restrita a `readOnly:` não encontrou nenhuma ocorrência. O recorte de cada chamada recuperou 3 seções (1 editável) na UnB e 5 (1 editável) nas outras três instalações, sem seção indeterminada. É **inspeção estrutural em Python**, não execução do binário Rust. Handoff: [#851](https://github.com/franklinbaldo/ies/issues/851#issuecomment-6913817640).

**Consequência:** um parser tipado Rust não pode extrair a gravabilidade da `textarea`, de uma chave sem aspas nem de cardinalidade fixa. O resultado apoia o futuro leitor de oferta, mas a implementação só deve entrar numa PR quando houver consumidor real e regressão de HTML capturado; sem novo erro/classificador defensivo.

**Limite:** 14 requests POST históricos continuam sem corpo/cabeçalhos. Confrontos IES/anti-IES com o POST original, versão persistida, assinaturas e live: **NOT_RUN**; 0 GET/POST externos, nenhuma escrita. `sei.test` continua a única confirmação sintética. Raposa pessoal `64b89f35`, árvore `5793ea5e` validada por Prek 2/2 no Ifro; CI privado da casa com falha sem steps, **NOT_VERIFIED**. Cafezinho [mesa #22](https://github.com/franklinbaldo/cafezinho/issues/22) e [IES #388](https://github.com/franklinbaldo/ies/issues/388) consultados.
