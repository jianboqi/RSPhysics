function matlab_reference(source_dir, output_dir)
% Generate INFORM comparison spectra from the EUFAR teaching archive.
    if ~isfolder(output_dir)
        mkdir(output_dir);
    end
    here = fileparts(mfilename('fullpath'));
    inputs = fullfile(here, '..', '..', '..', 'crates', 'models', ...
        'vegetation', 'inform', 'fixtures', 'parameters.csv');
    parameters = readmatrix(inputs);
    work = tempname(output_dir);
    mkdir(work);
    cleanup = onCleanup(@() clean_reference(work));
    copyfile(fullfile(source_dir, '*.m'), work);
    files = dir(fullfile(work, 'sail_*.m'));
    for i = 1:numel(files)
        filename = fullfile(work, files(i).name);
        replace_text(filename, 's=s+(rtp.*sks+rtm', 's=s+(rtp.*sks-rtm');
    end
    main = fullfile(work, 'inform.m');
    replace_line(main, 'Fcd', 'Fcd=min(min(co,cs),co*cs+p*sqrt(co*(1-co)*cs*(1-cs)));');
    replace_line(main, 'Fcs', 'Fcs=co-Fcd;');
    replace_line(main, 'Fod', 'Fod=cs-Fcd;');
    replace_line(main, 'Fos', 'Fos=(1-co)*(1-cs)+(Fcd-co*cs);');
    addpath(work);

    global data
    dataspec
    wavelengths = data(:,1);
    bands = 1:numel(wavelengths);
    soil = 0.16 + 0.00009*(wavelengths-400) + 0.035*sin((wavelengths-400)/330);
    under = prospect(2, 50, 0.025, 0.025);
    for i = 1:size(parameters, 1)
        p = parameters(i,:);
        [r,s,b,c,co,C,G,rl,tl,ts,to] = inform(p(12:15), p(1:7), ...
            p(8), p(9), p(10), p(11), soil, bands);
        writematrix([wavelengths,soil,rl,tl,under(:,2:3),r,s,b,c, ...
            repmat(co,numel(wavelengths),1),C,G,ts,to], fullfile(output_dir,sprintf('case%d.csv',i)));
    end
    writematrix(data, fullfile(output_dir, 'coefficients.csv'));
    writematrix(parameters, fullfile(output_dir, 'parameters.csv'));
end

function replace_text(file, from, to)
    source = fileread(file);
    assert(contains(source, from), 'Unexpected reference file: %s', file);
    write_text(file, strrep(source, from, to));
end

function replace_line(file, name, replacement)
    source = fileread(file);
    pattern = ['(?m)^' name '=[^;]+;'];
    assert(~isempty(regexp(source, pattern, 'once')), 'Unexpected reference file: %s', file);
    write_text(file, regexprep(source, pattern, replacement));
end

function write_text(file, content)
    handle = fopen(file, 'w', 'n', 'UTF-8');
    assert(handle ~= -1, 'Could not write %s', file);
    close_file = onCleanup(@() fclose(handle));
    fprintf(handle, '%s', content);
end

function clean_reference(work)
    if contains(path, work)
        rmpath(work);
    end
    rmdir(work, 's');
end
