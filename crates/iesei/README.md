---
type: Guide
title: "iesei — entidades tipadas compartilhadas"
description: "Identidade de instalação e cobertura de observações para IES e anti-IES"
---

# iesei

A crate `iesei` reúne tipos sem rede, HTML, sessão nem banco de dados.
A extração da identidade antes em `ies-core` não muda o comportamento da
CLI `ies`. `Instalacao` e `VersaoSei` continuam únicos.

Uma `Cobertura::NaoObservada` não representa lista vazia. Uma tabela
observada carrega `total_declarado: Option<usize>` e
`faixa_observada: Option<FaixaObservada>`, mantendo a distinção entre
informação não publicada e zero confirmado.

Esta é a primeira fatia WIP da [#360](https://github.com/franklinbaldo/ies/issues/360).
A ingestão SQLite (#358), o servidor de Controle (#354) e o E2E da
CLI (#355, cenários AIES-01..04) ainda não foram integrados. Nenhuma
prova de compatibilidade anti-IES ou validação live é alegada.
