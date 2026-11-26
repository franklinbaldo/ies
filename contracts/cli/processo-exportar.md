---
type: Contract
title: Exportar processo
description: Geração nativa integral em PDF ou ZIP
---

# Exportação integral (#889)

`ies processo exportar NUP --formato pdf|zip --saida ARQUIVO`

- Abre a árvore conferida e usa somente a oferta de geração publicada nela.
- GET do formulário, um POST consultivo de Todos os documentos, GET do
  download publicado no iframe. Sem execução de JS, fallback ou retry.
- Exige mesmo processo, controlador e hash na forma observada.
- O destino local é explícito, exclusivo, com gravação privada em parcial.
  Confere HTTP 200, MIME, assinatura inicial e comprimento quando publicado;
  registra tamanho e SHA-256. Isso não equivale a validar estruturalmente
  todo PDF/ZIP nem a demonstrar completude documental.
- Ausência de oferta, sessão, formato, resposta ou arquivo válido é falha.
- Limite de bytes reutiliza IES_LIMITE_ANEXO_BYTES; espera do transporte atual
  permanece inalterada (Todos usa 300 s para geração).
- ZIP seletivo: `--documentos DOCUMENTO` repetível, Nº SEI ou ID interno
  exatos; a identidade deve ser única na árvore e não pode repetir.
  Seleção desconhecida falha antes do POST gerador. A modalidade A e o
  oculto hdnDocumentosApenas estão presentes no HTML original. A composição
  id±título com ¥ usa o primitivo de lupa e é hipótese baseada no Todos.
- Prova de completude da saída real permanece pendente. O E2E anti-IES
  testa identidade/bytes sintéticos, não o POST original nem paridade live.
