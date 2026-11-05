---
type: Decision
title: Blocos somente pela Web observada
description: Rota de leitura, identidade e semântica da página observada
---

# Decisão: blocos Web sem inventar contagem

O IES lê blocos pela rota do menu realmente oferecida no Controle do
usuário. Rejeita uma ação supostamente igual em outra origem ou com
parâmetros de identidade duplicados. A lista `tblBlocos` permite
publicar estado, descrição e número, mas **não** quantos documentos
ou processos cada bloco guarda: a saída conserva
`quantidade_itens: null` até haver observação de detalhe.

O Todos @323eb01 é oráculo de cenário, não fonte de arquitetura:
`src/todos/sei/blocos_lista.py` explica diferenças de colunas
(assinatura, interno, reunião), `frmBlocoLista` e
`tblProtocolosBlocos`. A lista de blocos do SEI é paginada e
usa filtro persistente. Por isso, até existir derivação controlada
e testada da navegação, `--pagina N` maior que 1 falha
antes de acessar o SEI. O comando sem página preserva e informa
a página observada, sem restaurar filtros e sem chamar JavaScript.

Este recorte implementa apenas leituras. Escritas em SEI, mesmo de
treinamento, não são necessárias. Segurança prevalece sobre
publicar uma lista de cobertura desconhecida.
