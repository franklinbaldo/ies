---
type: ImplementationDecision
title: Processo marcar por id e nome com leitura completa e escrita fechada até formulário observado
operation: processo.marcar
implementation: partial
status: adopted-for-scope
scope: interpretacao da identidade id#nome e previa de retirada, com atribuicao idempotente e retirada idempotente; sem POST remoto
selected_strategy: reaproveitar-leitor-web-e-falhar-fechado-sem-formulario
alternatives: reaproveitar-leitor-web-e-falhar-fechado-sem-formulario,post-inferido-da-tela,usar-api-rest
evidence_level: estrutura-observada-unb-e-cassettes-de-escrita
live_validation: not-run-sandbox-sem-acesso-sei
---

# Decisão: `processo marcar` (Refs #255)

## Evidência

A [tela observada na UnB](../como-o-sei-funciona/marcadores-do-processo.md)
e o relatório de [leitura live](../validacao-live/2026-11-04-processo-obter-marcadores.md)
demonstram que o mesmo `id_marcador` aparece em processos distintos; o
rótulo `id#nome` não é identidade da associação ao processo. O
`tblMarcadores` declara seu total, mas pode publicar menos linhas, por
isso a ausência só é comprovada numa lista completa. Os botões de
adição/remoção e a ação servida ainda exigem evidência de POST.

## Alternativas

1. **Reusar `processo_obter`** para conferir unidade, identidade e
   marcadores, devolver prévia ou idempotência e falhar fechado quando
   precisaria escrever (escolhida neste recorte).
2. Inferir e enviar o POST a partir de `acaoRemover`, sem conferir o
   catálogo da unidade e sem observar a resposta: rejeitada, pois
   arriscaria remoção sem verificação e sem prova do formato servido.
3. REST/SOAP: não é a via da CLI nem foi observado como equivalente.

## Regras

Antes de decisão, validar número e rótulo sem arquivo ou rede.
Presença: id exato e nome exato; nome divergente é recusa, não
substituição. Ausência: `total_declarado == itens.len()`, nunca `null`
como zero. Retirar existente produz só prévia; se `--confirmar`, falha
tipada e **não** envia nada. O transporte, cookie e índice pertencem às
peças já existentes; nenhuma tela é restaurada.

## Revisão e limitações

A próxima fatia precisa observar formulário e opções **da unidade**,
derivar um único envio pelo `formulario_web` e provar o resultado numa
nova leitura. Se a confirmação for ambígua, saída 3 e nenhuma repetição.
Revisitar esta decisão após replay dos cenários `marcador-escritas-*`
e validação live em treinamento. Nada foi escrito no SEI nesta fatia.
