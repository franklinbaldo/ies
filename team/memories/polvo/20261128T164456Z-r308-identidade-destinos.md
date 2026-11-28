---
type: AgentMemory
title: "Polvo R308 — identidade dos destinos"
description: "Formulário e lupa não comprovam oferta; vincular os destinos ao processo."
agent: polvo
timestamp: "2026-11-28T16:44:56Z"
tags: [bloco, processo, formulario]
---

# Retomar

Duas páginas reais sanitizadas do Todos @caabe699 ([nó documental](https://github.com/franklinbaldo/todos/blob/caabe699bcf13b61b12dc5863638c8d90c0155c1/tests/paginas_reais/arvore_visualizar_documento.html), [aceite de criação](https://github.com/franklinbaldo/todos/blob/caabe699bcf13b61b12dc5863638c8d90c0155c1/tests/paginas_reais/documento_gerar_aceite.html)) têm frmVisualizar, dois controles, action e lupa, mas não têm a oferta literal incluirEmBloco(). Não confundir formulário com permissão. Destinos têm id_procedimento coincidente e único (2/2). Modelo JS 16/16 mutantes de identidade recusados; não é Rust nem replay original. Reutilizar ConsultaDoControlador::unico para ligar ambos os destinos ao ID durável da árvore; primitivo de formulário confere host, ação e hash, não esse ID. txtBloco usuário; hdnIdBloco oculto exigido e reescrito; demais ocultos frescos preservados. #881 a4ca70e1 rápida verde; dez paths inversos apenas memórias, mergeable=false. #913 READY/G379, preservar #913→#923. Rust/E2E/replay/prek/live NOT_RUN. [Recibo #881](https://github.com/franklinbaldo/ies/pull/881#issuecomment-7242506050).
