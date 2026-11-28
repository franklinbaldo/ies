---
type: Contract
title: Inventário offline de referências documentais e processuais
description: Proposta experimental de identificação lexical de candidatos no HTML editorial local
---

# documento referencias (proposta R320)

`ies [--json] documento referencias --arquivo HTML [--proprio DOC] [--nup NUP]`

Lê um arquivo privado local UTF-8 (limite já usado pelo leitor HTML). Não abre sessão, não usa índice, não consulta SEI, não modifica arquivo nem integra o fluxo de salvar. `--proprio` e `--nup` excluem grafias literais informadas, sem verificar identidade. Saída JSON: `escopo: texto_extraido_offline` e `candidatos`, cada um com tipo (documento/processo), texto, ocorrencias e estado candidato. Sem IDs, offsets ou vínculo confirmado. Agrupa por tipo e grafia na ordem da primeira ocorrência; preserva zeros.

Recorte R319/R320: dígitos ASCII; documentos de oito dígitos ou dez começando por 00 e NUP no formato observado de quatro/cinco dígitos, ponto, seis dígitos, barra, quatro dígitos, hífen e dois dígitos. Letras/números Unicode, marcas combinantes e underscore adjacentes impedem extrair de um token. Contextos alheios (CPF, PJe etc.) excluídos pelo leitor. Não normaliza grafia. HTML5 do fragmento editorial, texto de nós inline e entidades; separação nos blocos e subárvores excluídas (âncoras, head/title, script/style/template, controles, noscript, atributo hidden por presença). Não interpreta CSS: não é texto renderizado. Não promete identificar todos os documentos reais ou reescrever o HTML.

Sem captura SEI aplicável à operação inteiramente offline; matriz sintética R310 e contraprovas R311 preservadas. A proposta permanece DRAFT até revisão e CI.
